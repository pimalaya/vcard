//! # Decode (syntax to model)
//!
//! The read side of the bridge: project a raw syntax tree onto the decoded
//! model.
//!
//! `unescape` is the read codec (it resolves the RFC 6350 3.4 value escapes).
//! On top of it sit the `decode` methods, one per syntactic node: a
//! [`VcardValueNode`] decodes its components, a [`VcardParamNode`] decodes into
//! a [`VcardParam`], a [`VcardLine`] decodes into a [`VcardProp`], and a
//! [`VcardCst`] decodes into a whole [`Vcard`]. A property's value kind is no
//! longer chosen by a name match: [`VcardLine::decode`] resolves the prop name
//! to a [`VcardPropKind`], asks its spec for the in-force value kind (version
//! plus any declared `VALUE`), then routes to that kind's decoder. The
//! parameter name dispatch is still the match in [`VcardParamNode::decode`].
//! Each decoded value type gets an inherent `decode` that both paths delegate
//! to.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::{
    param::{VcardParam, VcardParamKind},
    prop::{VcardProp, VcardPropKind, VcardPropName},
    tree::{
        codec::Escaper, cst::VcardCst, line::VcardLine, param::VcardParamNode, prop::prop_spec,
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
        let qp = self.is_quoted_printable();
        let params = self
            .params
            .iter()
            .filter(|param| !param_is_quoted_printable(param))
            .map(VcardParamNode::decode)
            .collect();

        let value = match name.parse::<VcardPropKind>() {
            Ok(prop) => self.decode_value(prop, version),
            Err(_) => VcardValue::Unknown(VcardUnknownValue {
                components: self
                    .value
                    .components
                    .iter()
                    .map(|component| {
                        component
                            .iter()
                            .map(|v| unescape_with(v.get(), self.value.escaper))
                            .collect()
                    })
                    .collect(),
            }),
        };

        // NOTE: QUOTED-PRINTABLE values carry their octets in the wire text;
        // decode them on read so the model holds clean text, and drop the
        // now-stale encoding parameter (filtered above). Param-driven,
        // version-agnostic.
        let value = if qp { qp_decode_value(value) } else { value };

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
        decode_value_kind(kind, &self.value, version)
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
    fn is_quoted_printable(&self) -> bool {
        self.params.iter().any(param_is_quoted_printable)
    }
}

/// Decode a value node as the given value kind, branching on the version only
/// where the value's wire shape is version-specific (the `GEO` pair separator).
fn decode_value_kind<'v>(
    kind: VcardValueKind,
    node: &'v VcardValueNode<'_>,
    version: VcardVersion,
) -> VcardValue<'v> {
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
        VcardValueKind::Geo => VcardValue::Geo(decode_geo_pair(node, version)),
        // NOTE: base64 kept verbatim; a 2.1 / 3.0 URI reference is reached via
        // `VALUE=uri`, which resolves to the `Uri` kind above, not here.
        VcardValueKind::Binary => VcardValue::Binary(VcardBinary::Base64(node.decode_scalar_at(0))),
    }
}

/// Decode a `GEO` coordinate pair: `,`-separated in 2.1, `;`-separated
/// otherwise.
fn decode_geo_pair<'v>(node: &'v VcardValueNode<'_>, version: VcardVersion) -> VcardGeo<'v> {
    match version {
        VcardVersion::V2_1 => VcardGeo::decode_comma(node),
        _ => VcardGeo::decode_pair(node),
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
        let Ok(kind) = self.name.get().parse::<VcardParamKind>() else {
            return VcardParam::Unknown(unescape(self.name.get()), self.list());
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

impl VcardUtcOffset<'_> {
    /// Decode a UTC-offset value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardUtcOffset<'v> {
        VcardUtcOffset(node.decode_scalar_at(0))
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
        Escaper::V2_1 => unescape_v21(text),
    }
}

/// Resolve value escapes with the modern (RFC 2426 / 6350) rules. The default
/// used wherever the escaping mode is not version-specific (parameters, the
/// version-blind lens path).
pub(crate) fn unescape(text: &str) -> Cow<'_, str> {
    unescape_modern(text)
}

/// Resolve the RFC 2426 / 6350 3.4 value escapes `\\` `\,` `\;` `\n`, borrowing
/// when there is nothing to unescape.
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
