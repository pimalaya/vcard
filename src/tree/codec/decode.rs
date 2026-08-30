//! # Decode (syntax to model)
//!
//! The read side of the structural bridge: project a raw syntax tree onto the
//! decoded model. A [`VcardValueNode`] decodes its components, a
//! [`VcardParamNode`] decodes into a [`VcardParam`], a [`VcardLine`] into a
//! [`VcardProp`], and a [`VcardCst`] into a whole [`Vcard`].
//!
//! A property's value kind is resolved through its spec, not a name match:
//! [`VcardLine::decode`] maps the name to a [`VcardPropKind`], asks the spec
//! for the in-force value kind (version plus any declared `VALUE`), then routes
//! to that kind's decoder.
//!
//! Value escapes are resolved by the sibling
//! [`unescape`](crate::tree::codec::unescape) codec; content transfer encodings
//! (`QUOTED-PRINTABLE`, `BASE64`) and `CHARSET` are left to the feature
//! helpers.

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    param::{VcardParam, VcardParamKind},
    prop::{VcardProp, VcardPropKind, VcardPropName, spec::prop_spec},
    tree::{
        codec::{VcardCodec, unescape::unescape_param},
        cst::VcardCst,
        line::VcardLine,
        param::node::VcardParamNode,
        value::node::VcardValueNode,
    },
    value::{
        VcardValue, VcardValueKind, VcardValueUnknown,
        adr::VcardAdr,
        binary::VcardBinary,
        client_pid_map::VcardClientPidMap,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        gender::VcardGender,
        geo::VcardGeo,
        language::VcardLanguageTag,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
        utc_offset::VcardUtcOffset,
    },
    vcard::Vcard,
    version::VcardVersion,
};

impl VcardCst<'_> {
    /// Decode the whole card into the semantic [`Vcard`] model.
    pub fn decode(&self) -> Vcard<'_> {
        let version = self.version();

        // NOTE: VERSION is held as the card's indicator, not as a free
        // property.
        let properties = self
            .props
            .iter()
            .filter(|line| !line.name.get().eq_ignore_ascii_case("VERSION"))
            .map(|line| line.decode(version))
            .collect();

        Vcard {
            version,
            properties,
        }
    }
}

impl VcardLine<'_> {
    /// Decode the line into a typed property. A known property dispatches its
    /// value through the spec (see `decode_value`); an unknown one keeps its
    /// raw components so it round-trips.
    pub fn decode(&self, version: VcardVersion) -> VcardProp<'_> {
        let name = self.name.get();
        let params = self.params.iter().map(VcardParamNode::decode).collect();

        let value = match name.parse::<VcardPropKind>() {
            Ok(prop) => self.decode_value(prop, version),
            Err(_) => VcardValue::Unknown(VcardValueUnknown::decode(&self.value)),
        };

        VcardProp {
            name: VcardPropName::from(name),
            params,
            value,
        }
    }

    /// Decode a known property's value through its spec: resolve the in-force
    /// value kind from the card version and any declared `VALUE`, then run that
    /// kind's decoder over the value node. Shared by the whole-card decode and
    /// the version-specific lenses (`GEO`, the binary props).
    pub(crate) fn decode_value(
        &self,
        prop: VcardPropKind,
        version: VcardVersion,
    ) -> VcardValue<'_> {
        let declared = self.declared_value_kind();
        let kind = (prop_spec(prop).value)(version, declared);
        decode_value_kind(kind, &self.value)
    }

    /// The value kind named by this line's `VALUE` parameter, if any. Only the
    /// declared kind selects the value type; `ENCODING` / `CHARSET` transform
    /// the text and stay in the codec.
    fn declared_value_kind(&self) -> Option<VcardValueKind> {
        self.params
            .iter()
            .find(|param| matches!(param.name.get().parse(), Ok(VcardParamKind::Value)))
            .and_then(|param| param.values.first())
            .and_then(|value| value.get().parse::<VcardValueKind>().ok())
    }

    /// Whether the line declares the `QUOTED-PRINTABLE` encoding, as an
    /// `ENCODING=` parameter or a bare token (the 2.1 short form).
    #[cfg(feature = "quoted-printable")]
    pub(crate) fn is_quoted_printable(&self) -> bool {
        self.params.iter().any(param_is_quoted_printable)
    }

    /// The value of this line's `CHARSET` parameter, if any.
    #[cfg(feature = "encoding")]
    pub(crate) fn charset_label(&self) -> Option<&str> {
        self.params
            .iter()
            .find(|param| param.name.get().eq_ignore_ascii_case("CHARSET"))
            .and_then(|param| param.values.first())
            .map(|value| value.get())
    }
}

/// Decode a value node as the given value kind, routing to that value type's
/// [`VcardCodec`]. No version is needed: the one version-specific shape (the
/// `GEO` pair separator) is resolved from the node's escaper inside its codec.
fn decode_value_kind<'v>(kind: VcardValueKind, node: &'v VcardValueNode<'_>) -> VcardValue<'v> {
    match kind {
        VcardValueKind::Text => VcardValue::Text(VcardText::decode(node)),
        VcardValueKind::TextList => VcardValue::TextList(VcardTextList::decode(node)),
        VcardValueKind::Uri => VcardValue::Uri(VcardUri::decode(node)),
        VcardValueKind::DateAndOrTime => {
            VcardValue::DateAndOrTime(VcardDateAndOrTime::decode(node))
        }
        VcardValueKind::Timestamp => VcardValue::Timestamp(VcardTimestamp::decode(node)),
        VcardValueKind::LanguageTag => VcardValue::LanguageTag(VcardLanguageTag::decode(node)),
        VcardValueKind::UtcOffset => VcardValue::UtcOffset(VcardUtcOffset::decode(node)),
        VcardValueKind::N => VcardValue::N(VcardN::decode(node)),
        VcardValueKind::Adr => VcardValue::Adr(VcardAdr::decode(node)),
        VcardValueKind::Gender => VcardValue::Gender(VcardGender::decode(node)),
        VcardValueKind::Org => VcardValue::Org(VcardOrg::decode(node)),
        VcardValueKind::ClientPidMap => VcardValue::ClientPidMap(VcardClientPidMap::decode(node)),
        VcardValueKind::Geo => VcardValue::Geo(VcardGeo::decode(node)),
        VcardValueKind::Binary => VcardValue::Binary(VcardBinary::decode(node)),
    }
}

impl VcardParamNode<'_> {
    /// Decode the parameter into a typed parameter, dispatching on the name.
    pub fn decode(&self) -> VcardParam<'_> {
        let Ok(kind) = self.name.get().parse::<VcardParamKind>() else {
            return VcardParam::Unknown {
                // NOTE: a parameter name is a token (RFC 6350 3.3), with no
                // encoding of any kind to resolve.
                name: Cow::Borrowed(self.name.get()),
                values: self.list(),
            };
        };

        match kind {
            VcardParamKind::Language => VcardParam::Language(self.scalar()),
            VcardParamKind::Charset => VcardParam::Charset(self.scalar()),
            VcardParamKind::Encoding => VcardParam::Encoding(self.scalar()),
            VcardParamKind::Value => VcardParam::Value(self.scalar()),
            VcardParamKind::Pref => VcardParam::Pref(self.scalar()),
            VcardParamKind::AltId => VcardParam::AltId(self.scalar()),
            VcardParamKind::Pid => VcardParam::Pid(self.list()),
            VcardParamKind::Type => VcardParam::Type(self.list()),
            VcardParamKind::MediaType => VcardParam::MediaType(self.scalar()),
            VcardParamKind::CalScale => VcardParam::CalScale(self.scalar()),
            VcardParamKind::SortAs => VcardParam::SortAs(self.list()),
            VcardParamKind::Geo => VcardParam::Geo(self.scalar()),
            VcardParamKind::Tz => VcardParam::Tz(self.scalar()),
            VcardParamKind::Label => VcardParam::Label(self.scalar()),
            VcardParamKind::Author => VcardParam::Author(self.scalar()),
            VcardParamKind::AuthorName => VcardParam::AuthorName(self.scalar()),
            VcardParamKind::Created => VcardParam::Created(self.scalar()),
            VcardParamKind::Derived => VcardParam::Derived(self.scalar()),
            VcardParamKind::Jsptr => VcardParam::Jsptr(self.scalar()),
            VcardParamKind::Phonetic => VcardParam::Phonetic(self.scalar()),
            VcardParamKind::PropId => VcardParam::PropId(self.scalar()),
            VcardParamKind::Script => VcardParam::Script(self.scalar()),
            VcardParamKind::ServiceType => VcardParam::ServiceType(self.scalar()),
            VcardParamKind::Username => VcardParam::Username(self.scalar()),
        }
    }

    /// The parameter's first value, decoded by the RFC 6868 rules (empty when
    /// there is none).
    fn scalar(&self) -> Cow<'_, str> {
        self.values
            .first()
            .map(|v| unescape_param(v.get(), self.escaper))
            .unwrap_or(Cow::Borrowed(""))
    }

    /// The parameter's values, decoded by the RFC 6868 rules.
    fn list(&self) -> Vec<Cow<'_, str>> {
        self.values
            .iter()
            .map(|v| unescape_param(v.get(), self.escaper))
            .collect()
    }
}

/// Whether a parameter is `ENCODING=QUOTED-PRINTABLE` or the bare 2.1 token.
#[cfg(feature = "quoted-printable")]
fn param_is_quoted_printable(param: &VcardParamNode<'_>) -> bool {
    let name = param.name.get();

    (name.eq_ignore_ascii_case("ENCODING")
        && param
            .values
            .iter()
            .any(|v| v.get().eq_ignore_ascii_case("QUOTED-PRINTABLE")))
        || (param.values.is_empty() && name.eq_ignore_ascii_case("QUOTED-PRINTABLE"))
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::{
        param::VcardParam,
        tree::{
            codec::VcardCodec, cst::VcardCst, param::node::VcardParamNode,
            value::node::VcardValueNode,
        },
        value::{
            VcardValue,
            binary::VcardBinary,
            client_pid_map::VcardClientPidMap,
            datetime::{VcardDateAndOrTime, VcardTimestamp},
            gender::VcardGender,
            geo::VcardGeo,
            language::VcardLanguageTag,
            n::VcardN,
            org::VcardOrg,
            text::{VcardText, VcardTextList},
            uri::VcardUri,
            utc_offset::VcardUtcOffset,
        },
    };

    #[test]
    fn types_charset_and_encoding_params() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:2.1\r\n",
            "PHOTO;CHARSET=UTF-8;ENCODING=BASE64:Zm9v\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode();
        let params = &card.properties[0].params;

        assert!(params.contains(&VcardParam::Charset(Cow::Borrowed("UTF-8"))));
        assert!(params.contains(&VcardParam::Encoding(Cow::Borrowed("BASE64"))));
    }

    /// BDAY defaults to a date and PHOTO to inline base64 in 2.1, but a
    /// declared VALUE forces the other reading.
    #[test]
    fn the_value_param_selects_the_value_kind() {
        let cst = VcardCst::parse(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nBDAY;VALUE=text:circa 1800\r\nEND:VCARD\r\n",
        )
        .unwrap();
        assert_eq!(
            cst.decode().properties[0].value,
            VcardValue::Text(VcardText(Cow::Borrowed("circa 1800"))),
        );

        let cst = VcardCst::parse(
            "BEGIN:VCARD\r\nVERSION:2.1\r\nPHOTO;VALUE=URI:http://x/p.png\r\nEND:VCARD\r\n",
        )
        .unwrap();
        assert_eq!(
            cst.decode().properties[0].value,
            VcardValue::Uri(VcardUri(Cow::Borrowed("http://x/p.png"))),
        );
    }

    #[test]
    fn branches_geo_and_binary_on_version() {
        // NOTE: 2.1: GEO is a comma pair; PHOTO is inline base64.
        let v21 = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:2.1\r\n",
            "GEO:37.0,-122.0\r\n",
            "PHOTO;ENCODING=BASE64:Zm9v\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(v21).unwrap();
        let card = cst.decode();
        assert_eq!(
            card.properties[0].value,
            VcardValue::Geo(VcardGeo {
                latitude: Cow::Borrowed("37.0"),
                longitude: Cow::Borrowed("-122.0"),
            }),
        );
        assert_eq!(
            card.properties[1].value,
            VcardValue::Binary(VcardBinary::Base64(Cow::Borrowed("Zm9v"))),
        );

        // NOTE: 3.0: GEO is a semicolon pair.
        let v30 = "BEGIN:VCARD\r\nVERSION:3.0\r\nGEO:37.0;-122.0\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(v30).unwrap();
        let card = cst.decode();
        assert_eq!(
            card.properties[0].value,
            VcardValue::Geo(VcardGeo {
                latitude: Cow::Borrowed("37.0"),
                longitude: Cow::Borrowed("-122.0"),
            }),
        );

        // NOTE: 4.0: GEO is a URI; its comma is literal, so it is not
        // truncated.
        let v40 = "BEGIN:VCARD\r\nVERSION:4.0\r\nGEO:geo:37.0,-122.0\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(v40).unwrap();
        let card = cst.decode();
        assert_eq!(
            card.properties[0].value,
            VcardValue::Uri(VcardUri(Cow::Borrowed("geo:37.0,-122.0"))),
        );
    }

    /// The version-specific value shapes decode() resolves must come back
    /// identically through the typed lens, not as a version-blind URI.
    #[test]
    fn geo_and_binary_lenses_agree_with_whole_card_decode() {
        use crate::prop::{geo::GEO, photo::PHOTO};

        for input in [
            concat!(
                "BEGIN:VCARD\r\n",
                "VERSION:2.1\r\n",
                "GEO:37.0,-122.0\r\n",
                "PHOTO;ENCODING=BASE64:Zm9v\r\n",
                "END:VCARD\r\n",
            ),
            concat!(
                "BEGIN:VCARD\r\n",
                "VERSION:4.0\r\n",
                "GEO:geo:37.0,-122.0\r\n",
                "PHOTO:https://example.com/p.png\r\n",
                "END:VCARD\r\n",
            ),
        ] {
            let cst = VcardCst::parse(input).unwrap();
            let card = cst.decode();

            assert_eq!(cst.prop::<GEO>(), Some(card.properties[0].value.clone()));
            assert_eq!(cst.prop::<PHOTO>(), Some(card.properties[1].value.clone()));
        }
    }

    #[test]
    fn types_legacy_text_properties_instead_of_unknown() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:3.0\r\n",
            "LABEL:123 Main St\r\n",
            "NAME:Acme\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode();

        assert_eq!(
            card.properties[0].value,
            VcardValue::Text(VcardText(Cow::Borrowed("123 Main St"))),
        );
        assert_eq!(
            card.properties[1].value,
            VcardValue::Text(VcardText(Cow::Borrowed("Acme"))),
        );
    }

    /// The core transforms no content: the `=XX` octets stay in the value and
    /// the ENCODING parameter is kept, so a consumer can decode through the
    /// `quoted-printable` feature helper.
    #[test]
    fn keeps_a_quoted_printable_value_and_its_encoding_param_undecoded() {
        use crate::param::VcardParam;

        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:2.1\r\n",
            "NOTE;ENCODING=QUOTED-PRINTABLE:caf=C3=A9\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode();
        let prop = &card.properties[0];

        assert_eq!(
            prop.value,
            VcardValue::Text(VcardText(Cow::Borrowed("caf=C3=A9"))),
        );
        assert!(
            prop.params
                .contains(&VcardParam::Encoding(Cow::Borrowed("QUOTED-PRINTABLE"))),
        );
    }

    #[test]
    fn applies_version_specific_escaping() {
        use crate::prop::note::NOTE;

        // NOTE: 2.1: only `\;` is an escape; `\n` stays a literal backslash-n.
        let cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE:a\\nb\\;c\r\nEND:VCARD\r\n")
            .unwrap();
        let card = cst.decode();
        assert_eq!(
            card.properties[0].value,
            VcardValue::Text(VcardText(Cow::Borrowed("a\\nb;c"))),
        );

        // NOTE: 3.0: `\n` is a newline.
        let cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:3.0\r\nNOTE:a\\nb\\;c\r\nEND:VCARD\r\n")
            .unwrap();
        let card = cst.decode();
        assert_eq!(
            card.properties[0].value,
            VcardValue::Text(VcardText(Cow::Borrowed("a\nb;c"))),
        );

        // NOTE: 2.1 in-place edit escapes only `;`, leaving `,` literal.
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE:x\r\nEND:VCARD\r\n").unwrap();
        card.prop_mut::<NOTE>().unwrap().set_text("a,b;c");
        assert!(
            card.to_string().contains("NOTE:a,b\\;c\r\n"),
            "{}",
            card.to_string(),
        );
    }

    #[test]
    fn decodes_the_whole_value_and_its_named_components() {
        let node = VcardValueNode::parse(b"a,b;c");

        // NOTE: The whole-value reads keep every separator the value carries,
        // while the component reads cut at the `;` they name.
        assert_eq!(node.decode(), Cow::Borrowed("a,b;c"));
        assert_eq!(
            node.decode_list(),
            vec![Cow::Borrowed("a"), Cow::Borrowed("b;c")],
        );
        assert_eq!(
            node.decode_component_list(0),
            vec![Cow::Borrowed("a"), Cow::Borrowed("b")],
        );
        assert_eq!(node.decode_component(1), Cow::Borrowed("c"));
        assert_eq!(node.decode_component(9), Cow::Borrowed(""));
    }

    #[test]
    fn decodes_the_structured_n_value() {
        let node = VcardValueNode::parse(b"Doe;John;;Dr.;");
        let n = VcardN::decode(&node);
        assert_eq!(n.family, vec![Cow::Borrowed("Doe")]);
        assert_eq!(n.given, vec![Cow::Borrowed("John")]);
        assert_eq!(n.suffixes, vec![Cow::Borrowed("")]);
    }

    /// A kind with no `;`-structure of its own is decoded whole.
    ///
    /// RFC 6350 3.4 has a text value escape a semicolon it means literally,
    /// and 4.2 gives a URI no structure at all, so an unescaped `;` is content
    /// in either. Reading one component at a time cut such values short.
    #[test]
    fn decodes_every_unstructured_kind_whole() {
        let node = VcardValueNode::parse(b"a;b,c");

        assert_eq!(VcardText::decode(&node).0, "a;b,c");
        assert_eq!(VcardUri::decode(&node).0, "a;b,c");
        assert_eq!(
            VcardBinary::decode(&node),
            VcardBinary::Base64(Cow::Borrowed("a;b,c")),
        );
        assert_eq!(VcardDateAndOrTime::decode(&node).0, "a;b,c");
        assert_eq!(VcardTimestamp::decode(&node).0, "a;b,c");
        assert_eq!(VcardLanguageTag::decode(&node).0, "a;b,c");
        assert_eq!(VcardUtcOffset::decode(&node).0, "a;b,c");

        // NOTE: A list value owns its commas and nothing else, so only they
        // separate.
        assert_eq!(
            VcardTextList::decode(&node).0,
            vec![Cow::Borrowed("a;b"), Cow::Borrowed("c")],
        );
    }

    /// A structured value's component keeps the commas inside it.
    ///
    /// Each of these components is a text or a URI, where a comma separates
    /// nothing, so reading only a component's first comma-piece truncated the
    /// value: a client URI lost its query, a gender identity its second half.
    #[test]
    fn decodes_a_structured_component_past_its_first_comma() {
        let node = VcardValueNode::parse(b"1;urn:uuid:a,b");
        assert_eq!(VcardClientPidMap::decode(&node).uri, "urn:uuid:a,b");

        let node = VcardValueNode::parse(b"M;woman,she");
        assert_eq!(VcardGender::decode(&node).identity, "woman,she");

        let node = VcardValueNode::parse(b"Ada,Inc;R&D");
        assert_eq!(
            VcardOrg::decode(&node).0,
            vec![Cow::Borrowed("Ada,Inc"), Cow::Borrowed("R&D")],
        );
    }

    /// RFC 6868 section 3.1 spells the three characters a parameter value
    /// cannot carry raw.
    #[test]
    fn decodes_the_rfc_6868_parameter_sequences() {
        let node = VcardParamNode::parse("LABEL=a^nb^^c^'d");

        assert_eq!(node.decode(), VcardParam::Label(Cow::Borrowed("a\nb^c\"d")));
    }

    /// RFC 6868 section 3.1 forbids reading any other caret sequence as an
    /// error, so the caret and what follows stay literal, and so does a
    /// trailing one.
    #[test]
    fn keeps_an_unknown_caret_sequence_in_a_parameter() {
        let node = VcardParamNode::parse("LABEL=a^xb^");

        assert_eq!(node.decode(), VcardParam::Label(Cow::Borrowed("a^xb^")));
    }

    /// RFC 6868 section 3.2 forbids backslash escaping in a parameter value,
    /// so a Windows path keeps its separators.
    #[test]
    fn keeps_a_backslash_in_a_parameter() {
        let node = VcardParamNode::parse(r"X-PATH=C:\temp\note.txt");

        assert_eq!(
            node.decode(),
            VcardParam::Unknown {
                name: Cow::Borrowed("X-PATH"),
                values: vec![Cow::Borrowed(r"C:\temp\note.txt")],
            },
        );
    }

    /// RFC 6868 updates RFC 6350 alone, so a 3.0 caret is a literal caret:
    /// only the version on the card tells the two readings apart.
    #[test]
    fn keeps_a_pre_4_0_parameter_caret_literal() {
        let raw = "BEGIN:VCARD\r\nVERSION:3.0\r\nADR;LABEL=a^nb:;;;;;;\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(raw).unwrap();

        assert_eq!(
            cst.decode().properties[0].params[0],
            VcardParam::Label(Cow::Borrowed("a^nb")),
        );
    }
}
