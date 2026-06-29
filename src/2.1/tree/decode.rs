//! # Decode (syntax to model)
//!
//! The read side of the bridge: project a raw syntax tree onto the decoded
//! model.
//!
//! `unescape` and `qp_decode` are the 2.1 read codec: the first resolves the
//! `\;` value escape, the second the `QUOTED-PRINTABLE` `=XX` octets. On top of
//! them [`VcardLine::cooked`] cooks a line's flat `;`-components into clean
//! values, and the `decode` methods sit above that: a [`VcardParamNode`] decodes
//! into a [`VcardParam`], a [`VcardLine`] decodes into a [`VcardProp`], and a
//! [`VcardCst`] decodes into a whole [`Vcard`]. The name dispatch (which value
//! kind a property decodes to) is the match in [`VcardLine::decode`], kept here
//! so the generic nodes and the decoded model both stay free of it. Each decoded
//! value type also gets an inherent `decode` that the per-name lens markers
//! delegate to.

use alloc::{borrow::Cow, string::String, vec, vec::Vec};

use crate::v21::{
    param::VcardParam,
    prop::VcardProp,
    tree::{cst::VcardCst, line::VcardLine, param::VcardParamNode},
    value::{
        VcardUnknownValue, VcardValue,
        adr::VcardAdr,
        binary::VcardBinary,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        geo::VcardGeo,
        n::VcardN,
        org::VcardOrg,
        text::VcardText,
        uri::VcardUri,
    },
    vcard::Vcard,
};
use crate::version::VcardVersion;

const QUOTED_PRINTABLE: &str = "QUOTED-PRINTABLE";

impl VcardCst<'_> {
    /// Decode the whole card into the semantic [`Vcard`] model.
    pub fn decode(&self) -> Vcard<'_> {
        Vcard {
            version: VcardVersion::from(Cow::Borrowed(self.version.raw_value())),
            properties: self.props.iter().map(VcardLine::decode).collect(),
        }
    }
}

impl VcardLine<'_> {
    /// Decode the line into a typed property, dispatching the value on the name.
    pub fn decode(&self) -> VcardProp<'_> {
        VcardProp {
            name: Cow::Borrowed(self.name.get()),
            params: self.decode_params(),
            value: self.decode_value(),
        }
    }

    /// The line's cooked values: each flat `;`-component, unescaped and (when the
    /// line is `QUOTED-PRINTABLE`) `=XX`-decoded. For the compound values
    /// (`N`, `ADR`, `ORG`), whose `;` *is* a separator.
    pub fn cooked(&self) -> Vec<Cow<'_, str>> {
        let qp = self.is_quoted_printable();

        self.value
            .components
            .iter()
            .map(|leaf| {
                let text = unescape(leaf.get());
                if qp { qp_decode(text) } else { text }
            })
            .collect()
    }

    /// The whole value as one cooked string, for the non-compound values: the
    /// `;`-components rejoined (in 2.1 the `;` is literal outside `N`/`ADR`/`ORG`),
    /// then `\;`-unescaped and, when `QUOTED-PRINTABLE`, `=XX`-decoded.
    pub fn cooked_value(&self) -> Cow<'_, str> {
        let qp = self.is_quoted_printable();

        let text = match self.raw_value_full() {
            Cow::Borrowed(raw) => unescape(raw),
            Cow::Owned(raw) => Cow::Owned(unescape(&raw).into_owned()),
        };

        if qp { qp_decode(text) } else { text }
    }

    /// Decode every parameter, merging all `TYPE` parameters into one.
    fn decode_params(&self) -> Vec<VcardParam<'_>> {
        let mut params: Vec<VcardParam> = Vec::new();

        for node in &self.params {
            match node.decode() {
                // the value is QP-decoded on read, so the now-stale encoding
                // parameter is dropped to keep the decoded model consistent.
                VcardParam::Encoding(encoding)
                    if encoding.eq_ignore_ascii_case(QUOTED_PRINTABLE) => {}
                VcardParam::Type(mut values) => {
                    if let Some(VcardParam::Type(existing)) =
                        params.iter_mut().find(|p| matches!(p, VcardParam::Type(_)))
                    {
                        existing.append(&mut values);
                    } else {
                        params.push(VcardParam::Type(values));
                    }
                }
                other => params.push(other),
            }
        }

        params
    }

    /// Decode the value, dispatching the kind on the property name.
    fn decode_value(&self) -> VcardValue<'_> {
        use crate::v21::prop::*;

        match self.name.get().to_ascii_uppercase().as_str() {
            VCARD_FN | VCARD_EMAIL | VCARD_TITLE | VCARD_ROLE | VCARD_NOTE | VCARD_TEL
            | VCARD_MAILER | VCARD_LABEL | VCARD_AGENT | VCARD_TZ | VCARD_UID => {
                VcardValue::Text(VcardText::decode(self))
            }
            VCARD_URL => VcardValue::Uri(VcardUri::decode(self)),
            VCARD_BDAY => VcardValue::DateAndOrTime(VcardDateAndOrTime::decode(self)),
            VCARD_REV => VcardValue::Timestamp(VcardTimestamp::decode(self)),
            VCARD_N => VcardValue::N(VcardN::decode(self)),
            VCARD_ADR => VcardValue::Adr(VcardAdr::decode(self)),
            VCARD_ORG => VcardValue::Org(VcardOrg::decode(self)),
            VCARD_GEO => VcardValue::Geo(VcardGeo::decode(self)),
            VCARD_PHOTO | VCARD_LOGO | VCARD_SOUND | VCARD_KEY => {
                VcardValue::Binary(VcardBinary::decode(self))
            }
            _ => VcardValue::Unknown(VcardUnknownValue {
                components: self.cooked(),
            }),
        }
    }

    /// Whether the line carries a `QUOTED-PRINTABLE` encoding, written as an
    /// `ENCODING=` parameter or as a bare 2.1 token.
    fn is_quoted_printable(&self) -> bool {
        self.params.iter().any(|p| {
            let name = p.name.get();
            (name.eq_ignore_ascii_case("ENCODING")
                && p.values
                    .iter()
                    .any(|v| v.get().eq_ignore_ascii_case(QUOTED_PRINTABLE)))
                || (p.values.is_empty() && name.eq_ignore_ascii_case(QUOTED_PRINTABLE))
        })
    }

    /// Whether the line declares its binary value to be an external URI, written
    /// as a `VALUE=URL` parameter or as a bare 2.1 token.
    fn is_uri_reference(&self) -> bool {
        self.params.iter().any(|p| {
            let name = p.name.get();
            (name.eq_ignore_ascii_case("VALUE")
                && p.values.iter().any(|v| {
                    let v = v.get();
                    v.eq_ignore_ascii_case("URL") || v.eq_ignore_ascii_case("URI")
                }))
                || (p.values.is_empty()
                    && (name.eq_ignore_ascii_case("URL") || name.eq_ignore_ascii_case("URI")))
        })
    }
}

impl VcardParamNode<'_> {
    /// Decode the parameter into a typed parameter, dispatching on the name. A
    /// bare token (no `=value`) is classified by `classify_bare`.
    pub fn decode(&self) -> VcardParam<'_> {
        use crate::v21::param::*;

        if self.values.is_empty() {
            return classify_bare(self.name.get());
        }

        match self.name.get().to_ascii_uppercase().as_str() {
            VCARD_CHARSET => VcardParam::Charset(self.scalar()),
            VCARD_ENCODING => VcardParam::Encoding(self.scalar()),
            VCARD_LANGUAGE => VcardParam::Language(self.scalar()),
            VCARD_VALUE => VcardParam::Value(self.scalar()),
            VCARD_TYPE => VcardParam::Type(self.list()),
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

impl VcardText<'_> {
    /// Decode a single text value from a content line.
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardText<'v> {
        VcardText(line.cooked_value())
    }
}

impl VcardUri<'_> {
    /// Decode a URI value from a content line.
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardUri<'v> {
        VcardUri(line.cooked_value())
    }
}

impl VcardDateAndOrTime<'_> {
    /// Decode a date-and-or-time value from a content line.
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardDateAndOrTime<'v> {
        VcardDateAndOrTime(line.cooked_value())
    }
}

impl VcardTimestamp<'_> {
    /// Decode a timestamp value from a content line.
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardTimestamp<'v> {
        VcardTimestamp(line.cooked_value())
    }
}

impl VcardN<'_> {
    /// Decode the structured N value from a content line.
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardN<'v> {
        let mut c = line.cooked().into_iter();

        VcardN {
            family: c.next().unwrap_or_default(),
            given: c.next().unwrap_or_default(),
            additional: c.next().unwrap_or_default(),
            prefix: c.next().unwrap_or_default(),
            suffix: c.next().unwrap_or_default(),
        }
    }
}

impl VcardAdr<'_> {
    /// Decode the structured ADR value from a content line.
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardAdr<'v> {
        let mut c = line.cooked().into_iter();

        VcardAdr {
            po_box: c.next().unwrap_or_default(),
            extended: c.next().unwrap_or_default(),
            street: c.next().unwrap_or_default(),
            locality: c.next().unwrap_or_default(),
            region: c.next().unwrap_or_default(),
            postal_code: c.next().unwrap_or_default(),
            country: c.next().unwrap_or_default(),
        }
    }
}

impl VcardGeo<'_> {
    /// Decode the GEO value from a content line. vCard 2.1 joins the latitude and
    /// longitude with a comma (the semicolon form arrived in 3.0).
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardGeo<'v> {
        let (latitude, longitude) = split_pair(line.cooked_value(), ',');

        VcardGeo {
            latitude,
            longitude,
        }
    }
}

impl VcardOrg<'_> {
    /// Decode the structured ORG value from a content line.
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardOrg<'v> {
        VcardOrg(line.cooked())
    }
}

impl VcardBinary<'_> {
    /// Decode a binary value from a content line: a URI reference when the line
    /// says so, otherwise inline base64.
    pub fn decode<'v>(line: &'v VcardLine<'_>) -> VcardBinary<'v> {
        let raw = line.raw_value_full();

        if line.is_uri_reference() {
            VcardBinary::Uri(raw)
        } else {
            VcardBinary::Base64(raw)
        }
    }
}

/// Split a value into two at the first `sep`, preserving the borrow.
fn split_pair(value: Cow<'_, str>, sep: char) -> (Cow<'_, str>, Cow<'_, str>) {
    match value {
        Cow::Borrowed(s) => match s.split_once(sep) {
            Some((a, b)) => (Cow::Borrowed(a), Cow::Borrowed(b)),
            None => (Cow::Borrowed(s), Cow::Borrowed("")),
        },
        Cow::Owned(s) => match s.split_once(sep) {
            Some((a, b)) => (Cow::Owned(a.into()), Cow::Owned(b.into())),
            None => (Cow::Owned(s), Cow::Borrowed("")),
        },
    }
}

/// Classify a bare 2.1 parameter token (one with no `=value`): encoding values
/// map to ENCODING, value-type tokens to VALUE, everything else to TYPE.
fn classify_bare(token: &str) -> VcardParam<'_> {
    match token.to_ascii_uppercase().as_str() {
        "7BIT" | "8BIT" | "QUOTED-PRINTABLE" | "BASE64" | "B" => {
            VcardParam::Encoding(unescape(token))
        }
        "URL" | "URI" | "CONTENT-ID" | "CID" | "VCARD" | "INLINE" | "TEXT" => {
            VcardParam::Value(unescape(token))
        }
        _ => VcardParam::Type(vec![unescape(token)]),
    }
}

/// Resolve the 2.1 value escape `\;` (only), borrowing when there is nothing to
/// unescape.
pub(crate) fn unescape(text: &str) -> Cow<'_, str> {
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

/// The value of a hex digit, or None.
fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::v21::{
        tree::{
            decode::{qp_decode, unescape},
            line::VcardLine,
        },
        value::{VcardValue, geo::VcardGeo, n::VcardN, text::VcardText},
    };

    #[test]
    fn unescapes_the_semicolon_escape_and_borrows_when_clean() {
        assert_eq!(unescape(r"a\;b"), "a;b");
        assert!(matches!(unescape("plain"), Cow::Borrowed("plain")));
    }

    #[test]
    fn decodes_quoted_printable_octets() {
        assert_eq!(qp_decode(Cow::Borrowed("=C3=A9")), "é");
    }

    #[test]
    fn decodes_the_structured_n_value() {
        let (line, _) = VcardLine::take("N:Doe;John;;Dr.;\r\n").unwrap();
        let n = VcardN::decode(&line);

        assert_eq!(n.family, "Doe");
        assert_eq!(n.given, "John");
        assert_eq!(n.prefix, "Dr.");
    }

    #[test]
    fn keeps_a_literal_semicolon_in_a_simple_text_value() {
        let (line, _) = VcardLine::take("NOTE:Hello; world\r\n").unwrap();
        assert_eq!(
            line.decode().value,
            VcardValue::Text(VcardText(Cow::Borrowed("Hello; world"))),
        );
    }

    #[test]
    fn drops_the_stale_quoted_printable_parameter_on_decode() {
        let (line, _) = VcardLine::take("NOTE;ENCODING=QUOTED-PRINTABLE:caf=C3=A9\r\n").unwrap();
        let prop = line.decode();

        assert!(prop.params.is_empty());
        assert_eq!(
            prop.value,
            VcardValue::Text(VcardText(Cow::Borrowed("café")))
        );
    }

    #[test]
    fn decodes_geo_as_a_comma_pair() {
        let (line, _) = VcardLine::take("GEO:37.0,-122.0\r\n").unwrap();
        let geo = VcardGeo::decode(&line);

        assert_eq!(geo.latitude, "37.0");
        assert_eq!(geo.longitude, "-122.0");
    }
}
