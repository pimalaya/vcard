//! # Decode (syntax to model)
//!
//! The read side of the structural bridge: project a raw syntax tree onto the
//! decoded model. A [`VcardValueNode`] decodes its components, a
//! [`VcardParamNode`] decodes into a [`VcardParam`], a [`VcardLine`] decodes
//! into a [`VcardProp`], and a [`VcardCst`] decodes into a whole [`Vcard`].
//!
//! A property's value kind is resolved through its spec, not a name match:
//! [`VcardLine::decode`] maps the name to a [`VcardPropKind`], asks the spec for
//! the in-force value kind (version plus any declared `VALUE`), then routes to
//! that kind's decoder. The parameter name dispatch is the match in
//! [`VcardParamNode::decode`]. Value escapes are resolved by the sibling
//! [`unescape`](crate::tree::codec::unescape) codec; content transfer encodings
//! (`QUOTED-PRINTABLE`, `BASE64`) and `CHARSET` are left to the feature helpers.

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    param::{VcardParam, VcardParamKind},
    prop::{VcardProp, VcardPropKind, VcardPropName},
    tree::{
        codec::{
            Codec,
            unescape::{unescape, unescape_bytes, unescape_with},
        },
        cst::VcardCst,
        line::VcardLine,
        param::VcardParamNode,
        prop::prop_spec,
        value::VcardValueNode,
    },
    value::{
        VcardUnknownValue, VcardValue, VcardValueKind,
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
            Err(_) => VcardValue::Unknown(VcardUnknownValue::decode(&self.value)),
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
/// [`Codec`]. No version is needed: the one version-specific shape (the `GEO`
/// pair separator) is resolved from the node's escaper inside its codec.
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
                name: unescape(self.name.get()),
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
        }
    }

    /// The parameter's first value, decoded (empty when there is none).
    fn scalar(&self) -> Cow<'_, str> {
        self.values
            .first()
            .map(|v| unescape(v.get()))
            .unwrap_or(Cow::Borrowed(""))
    }

    /// The parameter's values, decoded.
    fn list(&self) -> Vec<Cow<'_, str>> {
        self.values.iter().map(|v| unescape(v.get())).collect()
    }
}

impl VcardValueNode<'_> {
    /// Decode the `i`th component into a clean (unescaped) value list.
    pub fn decode_at(&self, i: usize) -> Vec<Cow<'_, str>> {
        self.components
            .get(i)
            .map(|leaves| {
                leaves
                    .iter()
                    .map(|leaf| unescape_with(leaf.as_bytes(), self.escaper))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The `i`th component's first value as raw unescaped bytes, not transcoded,
    /// for a value carrying a foreign charset.
    pub fn decode_bytes_at(&self, i: usize) -> Cow<'_, [u8]> {
        self.components
            .get(i)
            .and_then(|leaves| leaves.first())
            .map(|leaf| unescape_bytes(leaf.as_bytes(), self.escaper))
            .unwrap_or(Cow::Borrowed(b""))
    }

    /// Decode the `i`th component's first value (empty when there is none).
    pub fn decode_scalar_at(&self, i: usize) -> Cow<'_, str> {
        self.components
            .get(i)
            .and_then(|leaves| leaves.first())
            .map(|leaf| unescape_with(leaf.as_bytes(), self.escaper))
            .unwrap_or(Cow::Borrowed(""))
    }

    /// Decode the `i`th component as a single value, rejoining its
    /// `,`-separated pieces. For values like URIs whose comma is a literal part
    /// of the value, not a list separator (so they must not be truncated).
    pub fn decode_joined_at(&self, i: usize) -> Cow<'_, str> {
        let Some(leaves) = self.components.get(i) else {
            return Cow::Borrowed("");
        };

        if leaves.len() <= 1 {
            return self.decode_scalar_at(i);
        }

        let mut raw = Vec::new();
        for (j, leaf) in leaves.iter().enumerate() {
            if j > 0 {
                raw.push(b',');
            }
            raw.extend_from_slice(leaf.as_bytes());
        }

        Cow::Owned(unescape_with(&raw, self.escaper).into_owned())
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
        tree::{codec::Codec, cst::VcardCst, value::VcardValueNode},
        value::{
            VcardValue, binary::VcardBinary, geo::VcardGeo, n::VcardN, text::VcardText,
            uri::VcardUri,
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

    #[test]
    fn the_value_param_selects_the_value_kind() {
        // BDAY defaults to a date, but VALUE=text forces the text reading.
        let cst = VcardCst::parse(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nBDAY;VALUE=text:circa 1800\r\nEND:VCARD\r\n",
        )
        .unwrap();
        assert_eq!(
            cst.decode().properties[0].value,
            VcardValue::Text(VcardText(Cow::Borrowed("circa 1800"))),
        );

        // A 2.1 PHOTO is inline base64 by default, but a plain URI when the
        // line declares VALUE=uri (the old is_uri_reference path, now
        // spec-derived).
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
        // 2.1: GEO is a comma pair; PHOTO is inline base64.
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

        // 3.0: GEO is a semicolon pair.
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

        // 4.0: GEO is a URI; its comma is literal, so it is not truncated.
        let v40 = "BEGIN:VCARD\r\nVERSION:4.0\r\nGEO:geo:37.0,-122.0\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(v40).unwrap();
        let card = cst.decode();
        assert_eq!(
            card.properties[0].value,
            VcardValue::Uri(VcardUri(Cow::Borrowed("geo:37.0,-122.0"))),
        );
    }

    #[test]
    fn geo_and_binary_lenses_agree_with_whole_card_decode() {
        use crate::tree::prop::{geo::GEO, photo::PHOTO};

        // The version-specific value shapes that decode() resolves must come
        // back identically through the typed lens, not as a version-blind URI.
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

    #[test]
    fn keeps_a_quoted_printable_value_and_its_encoding_param_undecoded() {
        use crate::param::VcardParam;

        // Core transforms no content: the `=XX` octets stay in the value and the
        // ENCODING param is kept, so a consumer can decode via the
        // `quoted-printable` feature helper.
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
        use crate::tree::prop::note::NOTE;

        // 2.1: only `\;` is an escape; `\n` stays a literal backslash-n.
        let cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE:a\\nb\\;c\r\nEND:VCARD\r\n")
            .unwrap();
        let card = cst.decode();
        assert_eq!(
            card.properties[0].value,
            VcardValue::Text(VcardText(Cow::Borrowed("a\\nb;c"))),
        );

        // 3.0: `\n` is a newline.
        let cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:3.0\r\nNOTE:a\\nb\\;c\r\nEND:VCARD\r\n")
            .unwrap();
        let card = cst.decode();
        assert_eq!(
            card.properties[0].value,
            VcardValue::Text(VcardText(Cow::Borrowed("a\nb;c"))),
        );

        // 2.1 in-place edit escapes only `;`, leaving `,` literal.
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
    fn decodes_components_into_scalar_and_list() {
        let node = VcardValueNode::parse(b"a,b;c");
        assert_eq!(
            node.decode_at(0),
            vec![Cow::Borrowed("a"), Cow::Borrowed("b")],
        );
        assert_eq!(node.decode_scalar_at(1), Cow::Borrowed("c"));
        assert_eq!(node.decode_scalar_at(9), Cow::Borrowed(""));
    }

    #[test]
    fn decodes_the_structured_n_value() {
        let node = VcardValueNode::parse(b"Doe;John;;Dr.;");
        let n = VcardN::decode(&node);
        assert_eq!(n.family, vec![Cow::Borrowed("Doe")]);
        assert_eq!(n.given, vec![Cow::Borrowed("John")]);
        assert_eq!(n.suffixes, vec![Cow::Borrowed("")]);
    }
}
