//! # AGENT lens
//!
//! Reading and editing the `AGENT` property in place: it decodes as a
//! [`VcardText`] and edits through the generic [`VcardValueCursor`].
//!
//! A vCard 2.1 `AGENT` may embed a whole card in its value, which
//! [`VcardCst::agent`](crate::tree::cst::VcardCst::agent) reads back out.
//!
//! Its RFC contract sits on the marker, [`AGENT`].

use crate::{
    prop::agent::AGENT,
    tree::{
        cst::VcardCst, error::VcardParseError, line::VcardLine, prop::lens::VcardPropLens,
        value::cursor::VcardValueCursor,
    },
    value::text::VcardText,
};

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

impl VcardCst<'_> {
    /// Parse the vCard embedded in this card's `AGENT`, unescaped if 3.0.
    ///
    /// `AGENT` is opaque text, never decoded recursively to bound the work, so
    /// this is the opt-in for exactly one level. `None` without an `AGENT`, or
    /// when it embeds none: a URI, or a 2.1 agent whose lines are the card's.
    pub fn agent(&self) -> Option<Result<VcardCst<'static>, VcardParseError>> {
        let text = self.prop::<AGENT>()?;
        let bytes = text.0.into_owned().into_bytes();

        // NOTE: Only a BEGIN-wrapped value embeds a card; a URI reference or an
        // empty value does not (and parse() would read the former as a bare
        // record).
        let (first, _rest) = VcardLine::take(&bytes).ok()?;
        if !first.name.get().eq_ignore_ascii_case("BEGIN") {
            return None;
        }

        Some(VcardCst::parse(&bytes).map(VcardCst::into_static))
    }
}

#[cfg(test)]
mod tests {
    use crate::{prop::r#fn::FN, tree::cst::VcardCst};

    /// A 3.0 AGENT embedding a card, its newlines backslash-escaped.
    #[test]
    fn parses_the_embedded_agent_card() {
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
