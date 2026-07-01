//! # AGENT lens
//!
//! The `AGENT` property lens: another entity acting as the contact's agent,
//! kept as opaque text and decoded as a single text value.
//!
//! See RFC 2426 3.5.4 (vCard 3.0; removed in 4.0).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cst::VcardCst,
        error::VcardParseError,
        line::VcardLine,
        prop::{VcardPropCardinality, VcardPropLens, VcardPropSpec},
        value::VcardValueCursor,
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `AGENT` property lens.
pub struct AGENT;

impl VcardCst<'_> {
    /// Parse the vCard embedded in this card's `AGENT` property. `AGENT` is kept
    /// as opaque text and never decoded recursively (to bound the work), so a
    /// consumer opts into exactly one level here. The vCard 3.0 escaped form
    /// (newlines and delimiters backslash-escaped) is unescaped on read, then
    /// re-parsed.
    ///
    /// Returns `None` when there is no `AGENT`, or its value embeds no card: an
    /// `AGENT` URI reference, or the empty value of a 2.1 inline agent (whose
    /// embedded lines are separate properties of the outer card, not its value).
    pub fn agent(&self) -> Option<Result<VcardCst<'static>, VcardParseError>> {
        let text = self.prop::<AGENT>()?;
        let bytes = text.0.into_owned().into_bytes();

        // Only a BEGIN-wrapped value embeds a card; a URI reference or an empty
        // value does not (and parse() would read the former as a bare record).
        let (first, _rest) = VcardLine::take(&bytes).ok()?;
        if !first.name.get().eq_ignore_ascii_case("BEGIN") {
            return None;
        }

        Some(VcardCst::parse(&bytes).map(VcardCst::into_static))
    }
}

impl VcardPropLens for AGENT {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for AGENT {
    const KIND: VcardPropKind = VcardPropKind::Agent;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V2_1, VcardVersion::V3_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Language,
            VcardParamKind::Encoding,
            VcardParamKind::Charset,
            VcardParamKind::Value,
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::{cst::VcardCst, prop::r#fn::FN};

    #[test]
    fn parses_the_embedded_agent_card() {
        // A 3.0 AGENT embedding a card, its newlines backslash-escaped.
        let card = VcardCst::parse(concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:3.0\r\n",
            "FN:Boss\r\n",
            "AGENT:BEGIN:VCARD\\nVERSION:3.0\\nFN:Susan Thomas\\nEND:VCARD\\n\r\n",
            "END:VCARD\r\n",
        ))
        .unwrap();

        let agent = card.agent().expect("an AGENT property").expect("parses");
        assert_eq!(agent.prop::<FN>().unwrap().0, "Susan Thomas");
    }

    #[test]
    fn returns_none_when_agent_is_a_uri_reference() {
        let card = VcardCst::parse(concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:3.0\r\n",
            "AGENT;VALUE=uri:CID:JQPUBLIC.part3.960129T083020.xyzMail@example.com\r\n",
            "END:VCARD\r\n",
        ))
        .unwrap();

        assert!(card.agent().is_none());
    }
}
