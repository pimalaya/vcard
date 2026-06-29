//! # Decode (syntax to model)
//!
//! The read side of the bridge: project a raw syntax tree onto the decoded
//! model.
//!
//! `unescape` is the read codec (it resolves the RFC 6350 value escapes). On top
//! of it sit the `decode` methods, one per syntactic node: a
//! [`VcardValueNode`] decodes its components, a [`VcardParamNode`] decodes into a
//! [`VcardParam`], a [`VcardLine`] decodes into a [`VcardProp`], and a
//! [`VcardCst`] decodes into a whole [`Vcard`]. The name dispatch (which value
//! kind a property or parameter decodes to) is the match in
//! [`VcardLine::decode`] and [`VcardParamNode::decode`], kept here so the generic
//! nodes and the decoded model both stay free of it. Each decoded value type
//! also gets an inherent `decode` that the per-name lens markers delegate to.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::codec::Escaper;
use crate::version::{VCARD_VERSION, VcardVersion};
use crate::{
    param::VcardParam,
    prop::VcardProp,
    tree::{cst::VcardCst, line::VcardLine, param::VcardParamNode, value::VcardValueNode},
    value::{
        VcardUnknownValue, VcardValue,
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
};

impl VcardCst<'_> {
    /// Decode the whole card into the semantic [`Vcard`] model.
    pub fn decode(&self) -> Vcard<'_> {
        let version = self.version();

        // VERSION is held as the card's indicator, not as a free property.
        let properties = self
            .props
            .iter()
            .filter(|line| !line.name.get().eq_ignore_ascii_case(VCARD_VERSION))
            .map(|line| line.decode(&version))
            .collect();

        Vcard {
            version,
            properties,
        }
    }
}

impl VcardLine<'_> {
    /// Decode the line into a typed property, dispatching the value on the name.
    pub fn decode(&self, version: &VcardVersion<'_>) -> VcardProp<'_> {
        let name = self.name.get();
        let qp = self.is_quoted_printable();
        let params = self
            .params
            .iter()
            .filter(|param| !param_is_quoted_printable(param))
            .map(VcardParamNode::decode)
            .collect();
        let node = &self.value;

        let value = {
            use crate::prop::*;

            match name.to_ascii_uppercase().as_str() {
                VCARD_FN | VCARD_KIND | VCARD_XML | VCARD_TEL | VCARD_EMAIL | VCARD_TITLE
                | VCARD_ROLE | VCARD_NOTE | VCARD_PRODID | VCARD_TZ
                // 2.1 / 3.0 text properties with no 4.0 equivalent.
                | VCARD_LABEL | VCARD_NAME | VCARD_PROFILE | VCARD_MAILER | VCARD_AGENT
                | VCARD_CLASS | VCARD_SORT_STRING => {
                    VcardValue::Text(VcardText::decode(node))
                }
                VCARD_NICKNAME | VCARD_CATEGORIES => {
                    VcardValue::TextList(VcardTextList::decode(node))
                }
                VCARD_SOURCE | VCARD_IMPP | VCARD_MEMBER | VCARD_RELATED | VCARD_UID
                | VCARD_URL | VCARD_FBURL | VCARD_CALADRURI | VCARD_CALURI => {
                    VcardValue::Uri(VcardUri::decode(node))
                }
                VCARD_GEO => self.decode_geo(version),
                VCARD_PHOTO | VCARD_LOGO | VCARD_SOUND | VCARD_KEY => {
                    self.decode_binary_value(version)
                }
                VCARD_BDAY | VCARD_ANNIVERSARY => {
                    VcardValue::DateAndOrTime(VcardDateAndOrTime::decode(node))
                }
                VCARD_REV => VcardValue::Timestamp(VcardTimestamp::decode(node)),
                VCARD_LANG => VcardValue::LanguageTag(VcardLanguageTag::decode(node)),
                VCARD_N => VcardValue::N(VcardN::decode(node)),
                VCARD_ADR => VcardValue::Adr(VcardAdr::decode(node)),
                VCARD_GENDER => VcardValue::Gender(VcardGender::decode(node)),
                VCARD_ORG => VcardValue::Org(VcardOrg::decode(node)),
                VCARD_CLIENTPIDMAP => VcardValue::ClientPidMap(VcardClientPidMap::decode(node)),

                _ => VcardValue::Unknown(VcardUnknownValue {
                    components: node
                        .components
                        .iter()
                        .map(|component| {
                            component
                                .iter()
                                .map(|v| unescape_with(v.get(), node.escaper))
                                .collect()
                        })
                        .collect(),
                }),
            }
        };

        // QUOTED-PRINTABLE values carry their octets in the wire text; decode
        // them on read so the model holds clean text, and drop the now-stale
        // encoding parameter (filtered above). Param-driven, version-agnostic.
        let value = if qp { qp_decode_value(value) } else { value };

        VcardProp {
            name: Cow::Borrowed(name),
            params,
            value,
        }
    }

    /// Whether the line declares the `QUOTED-PRINTABLE` encoding, as an
    /// `ENCODING=` parameter or a bare token (the 2.1 short form).
    fn is_quoted_printable(&self) -> bool {
        self.params.iter().any(param_is_quoted_printable)
    }

    /// Decode the `GEO` value: a coordinate pair in 2.1 (`,`) and 3.0 (`;`), a
    /// URI in 4.0. Shared by the full-card decode and the `GEO` lens.
    pub(crate) fn decode_geo(&self, version: &VcardVersion<'_>) -> VcardValue<'_> {
        match version {
            VcardVersion::V21 => VcardValue::Geo(VcardGeo::decode_comma(&self.value)),
            VcardVersion::V30 => VcardValue::Geo(VcardGeo::decode_pair(&self.value)),
            _ => VcardValue::Uri(VcardUri::decode(&self.value)),
        }
    }

    /// Decode a binary-bearing value (`PHOTO`, `LOGO`, `SOUND`, `KEY`): inline
    /// base64 or a URI reference in 2.1 / 3.0, a `data:` URI in 4.0. Shared by
    /// the full-card decode and the binary lenses.
    pub(crate) fn decode_binary_value(&self, version: &VcardVersion<'_>) -> VcardValue<'_> {
        match version {
            VcardVersion::V21 | VcardVersion::V30 => VcardValue::Binary(self.decode_binary()),
            _ => VcardValue::Uri(VcardUri::decode(&self.value)),
        }
    }

    /// Decode a 2.1 / 3.0 binary value: a URI reference when the line says so
    /// (`VALUE=uri` or a bare token), otherwise inline base64 kept verbatim.
    fn decode_binary(&self) -> VcardBinary<'_> {
        let raw = self.value.decode_scalar_at(0);

        if self.is_uri_reference() {
            VcardBinary::Uri(raw)
        } else {
            VcardBinary::Base64(raw)
        }
    }

    /// Whether the line declares its value to be an external URI reference.
    fn is_uri_reference(&self) -> bool {
        self.params.iter().any(|param| {
            let name = param.name.get();

            (name.eq_ignore_ascii_case("VALUE")
                && param.values.iter().any(|v| {
                    let v = v.get();
                    v.eq_ignore_ascii_case("uri") || v.eq_ignore_ascii_case("url")
                }))
                || (param.values.is_empty()
                    && (name.eq_ignore_ascii_case("uri") || name.eq_ignore_ascii_case("url")))
        })
    }
}

impl VcardGeo<'_> {
    /// Decode a 3.0 `GEO` pair (`latitude;longitude`) from a syntax node.
    pub fn decode_pair<'v>(node: &'v VcardValueNode<'_>) -> VcardGeo<'v> {
        VcardGeo {
            latitude: node.decode_scalar_at(0),
            longitude: node.decode_scalar_at(1),
        }
    }

    /// Decode a 2.1 `GEO` pair (`latitude,longitude`) from a syntax node.
    pub fn decode_comma<'v>(node: &'v VcardValueNode<'_>) -> VcardGeo<'v> {
        let mut parts = node.decode_at(0).into_iter();

        VcardGeo {
            latitude: parts.next().unwrap_or_default(),
            longitude: parts.next().unwrap_or_default(),
        }
    }
}

impl VcardParamNode<'_> {
    /// Decode the parameter into a typed parameter, dispatching on the name.
    pub fn decode(&self) -> VcardParam<'_> {
        use crate::param::*;

        match self.name.get().to_ascii_uppercase().as_str() {
            VCARD_LANGUAGE => VcardParam::Language(self.scalar()),
            VCARD_CHARSET => VcardParam::Charset(self.scalar()),
            VCARD_ENCODING => VcardParam::Encoding(self.scalar()),
            VCARD_VALUE => VcardParam::Value(self.scalar()),
            VCARD_PREF => VcardParam::Pref(self.scalar()),
            VCARD_ALTID => VcardParam::AltId(self.scalar()),
            VCARD_PID => VcardParam::Pid(self.list()),
            VCARD_TYPE => VcardParam::Type(self.list()),
            VCARD_MEDIATYPE => VcardParam::MediaType(self.scalar()),
            VCARD_CALSCALE => VcardParam::CalScale(self.scalar()),
            VCARD_SORT_AS => VcardParam::SortAs(self.list()),
            VCARD_GEO => VcardParam::Geo(self.scalar()),
            VCARD_TZ => VcardParam::Tz(self.scalar()),
            VCARD_LABEL => VcardParam::Label(self.scalar()),

            _ => VcardParam::Unknown {
                name: unescape(self.name.get()),
                values: self.list(),
            },
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
                    .map(|leaf| unescape_with(leaf.get(), self.escaper))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Decode the `i`th component's first value (empty when there is none).
    pub fn decode_scalar_at(&self, i: usize) -> Cow<'_, str> {
        self.components
            .get(i)
            .and_then(|leaves| leaves.first())
            .map(|leaf| unescape_with(leaf.get(), self.escaper))
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

        let raw = leaves
            .iter()
            .map(|leaf| leaf.get())
            .collect::<Vec<_>>()
            .join(",");

        Cow::Owned(unescape_with(&raw, self.escaper).into_owned())
    }
}

impl VcardText<'_> {
    /// Decode a single text value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardText<'v> {
        VcardText(node.decode_scalar_at(0))
    }
}

impl VcardTextList<'_> {
    /// Decode a comma-separated text list from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardTextList<'v> {
        VcardTextList(node.decode_at(0))
    }
}

impl VcardUri<'_> {
    /// Decode a URI value from a syntax node. A URI's comma is literal, not a
    /// list separator, so the whole component is kept (not truncated at `,`).
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardUri<'v> {
        VcardUri(node.decode_joined_at(0))
    }
}

impl VcardDateAndOrTime<'_> {
    /// Decode a date-and-or-time value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardDateAndOrTime<'v> {
        VcardDateAndOrTime(node.decode_scalar_at(0))
    }
}

impl VcardTimestamp<'_> {
    /// Decode a timestamp value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardTimestamp<'v> {
        VcardTimestamp(node.decode_scalar_at(0))
    }
}

impl VcardLanguageTag<'_> {
    /// Decode a language-tag value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardLanguageTag<'v> {
        VcardLanguageTag(node.decode_scalar_at(0))
    }
}

impl VcardN<'_> {
    /// Decode the structured N value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardN<'v> {
        VcardN {
            family: node.decode_at(0),
            given: node.decode_at(1),
            additional: node.decode_at(2),
            prefixes: node.decode_at(3),
            suffixes: node.decode_at(4),
        }
    }
}

impl VcardAdr<'_> {
    /// Decode the structured ADR value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardAdr<'v> {
        VcardAdr {
            po_box: node.decode_at(0),
            extended: node.decode_at(1),
            street: node.decode_at(2),
            locality: node.decode_at(3),
            region: node.decode_at(4),
            postal_code: node.decode_at(5),
            country: node.decode_at(6),
        }
    }
}

impl VcardGender<'_> {
    /// Decode the structured GENDER value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardGender<'v> {
        VcardGender {
            sex: node.decode_scalar_at(0),
            identity: node.decode_scalar_at(1),
        }
    }
}

impl VcardOrg<'_> {
    /// Decode the structured ORG value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardOrg<'v> {
        let units = (0..node.components.len())
            .map(|i| node.decode_scalar_at(i))
            .collect();
        VcardOrg(units)
    }
}

impl VcardClientPidMap<'_> {
    /// Decode the structured CLIENTPIDMAP value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardClientPidMap<'v> {
        VcardClientPidMap {
            id: node.decode_scalar_at(0),
            uri: node.decode_scalar_at(1),
        }
    }
}

/// Resolve the RFC 6350 value escapes `\\` `\,` `\;` `\n`, borrowing when there
/// is nothing to unescape.
/// Whether a parameter is `ENCODING=QUOTED-PRINTABLE` or the bare 2.1 token.
fn param_is_quoted_printable(param: &VcardParamNode<'_>) -> bool {
    let name = param.name.get();

    (name.eq_ignore_ascii_case("ENCODING")
        && param
            .values
            .iter()
            .any(|v| v.get().eq_ignore_ascii_case("QUOTED-PRINTABLE")))
        || (param.values.is_empty() && name.eq_ignore_ascii_case("QUOTED-PRINTABLE"))
}

/// QUOTED-PRINTABLE-decode the text-bearing kinds of a decoded value, leaving
/// the structured kinds untouched (they rarely carry the encoding).
fn qp_decode_value(value: VcardValue<'_>) -> VcardValue<'_> {
    match value {
        VcardValue::Text(VcardText(c)) => VcardValue::Text(VcardText(qp_decode(c))),
        VcardValue::Uri(VcardUri(c)) => VcardValue::Uri(VcardUri(qp_decode(c))),
        VcardValue::DateAndOrTime(VcardDateAndOrTime(c)) => {
            VcardValue::DateAndOrTime(VcardDateAndOrTime(qp_decode(c)))
        }
        VcardValue::Timestamp(VcardTimestamp(c)) => {
            VcardValue::Timestamp(VcardTimestamp(qp_decode(c)))
        }
        VcardValue::LanguageTag(VcardLanguageTag(c)) => {
            VcardValue::LanguageTag(VcardLanguageTag(qp_decode(c)))
        }
        VcardValue::UtcOffset(VcardUtcOffset(c)) => {
            VcardValue::UtcOffset(VcardUtcOffset(qp_decode(c)))
        }
        VcardValue::TextList(VcardTextList(values)) => {
            VcardValue::TextList(VcardTextList(values.into_iter().map(qp_decode).collect()))
        }
        VcardValue::Unknown(VcardUnknownValue { components }) => {
            VcardValue::Unknown(VcardUnknownValue {
                components: components
                    .into_iter()
                    .map(|component| component.into_iter().map(qp_decode).collect())
                    .collect(),
            })
        }
        other => other,
    }
}

/// Decode QUOTED-PRINTABLE `=XX` octets (soft line breaks are already joined by
/// the tokeniser). Bytes are reassembled then read as UTF-8 (lossy).
pub(crate) fn qp_decode(input: Cow<'_, str>) -> Cow<'_, str> {
    if !input.as_bytes().contains(&b'=') {
        return input;
    }

    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'='
            && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2]))
        {
            out.push(hi << 4 | lo);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }

    Cow::Owned(String::from_utf8_lossy(&out).into_owned())
}

/// The value of a hex digit, or `None`.
fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

/// Resolve value escapes by the card's escaping mode.
pub(crate) fn unescape_with(text: &str, escaper: Escaper) -> Cow<'_, str> {
    match escaper {
        Escaper::Modern => unescape_modern(text),
        Escaper::V21 => unescape_v21(text),
    }
}

/// Resolve value escapes with the modern (RFC 2426 / 6350) rules. The default
/// used wherever the escaping mode is not version-specific (parameters, the
/// version-blind lens path).
pub(crate) fn unescape(text: &str) -> Cow<'_, str> {
    unescape_modern(text)
}

/// Resolve the RFC 2426 / 6350 value escapes `\\` `\,` `\;` `\n`, borrowing when
/// there is nothing to unescape.
fn unescape_modern(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }

    Cow::Owned(out)
}

/// Resolve the vCard 2.1 value escape `\;` only; a backslash before anything
/// else stays literal.
fn unescape_v21(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some(';') => out.push(';'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::{
        param::VcardParam,
        tree::{cst::VcardCst, decode::unescape, value::VcardValueNode},
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

        // The version-specific value shapes that decode() resolves must come back
        // identically through the typed lens, not as a version-blind URI.
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
    fn decodes_quoted_printable_value_and_drops_the_stale_encoding_param() {
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
            VcardValue::Text(VcardText(Cow::Borrowed("café")))
        );
        assert!(prop.params.is_empty(), "stale ENCODING param was kept");
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
    fn unescapes_value_escapes_and_borrows_when_clean() {
        assert_eq!(unescape(r"a\,b\;c\nd"), "a,b;c\nd");
        assert!(matches!(unescape("plain"), Cow::Borrowed("plain")));
    }

    #[test]
    fn decodes_components_into_scalar_and_list() {
        let node = VcardValueNode::parse("a,b;c");
        assert_eq!(
            node.decode_at(0),
            vec![Cow::Borrowed("a"), Cow::Borrowed("b")],
        );
        assert_eq!(node.decode_scalar_at(1), Cow::Borrowed("c"));
        assert_eq!(node.decode_scalar_at(9), Cow::Borrowed(""));
    }

    #[test]
    fn decodes_the_structured_n_value() {
        let node = VcardValueNode::parse("Doe;John;;Dr.;");
        let n = VcardN::decode(&node);
        assert_eq!(n.family, vec![Cow::Borrowed("Doe")]);
        assert_eq!(n.given, vec![Cow::Borrowed("John")]);
        assert_eq!(n.suffixes, vec![Cow::Borrowed("")]);
    }
}
