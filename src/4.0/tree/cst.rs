//! # Concrete syntax tree
//!
//! The core representation: a whole vCard as generic, byte-faithful syntax.
//!
//! [`VcardCst`] is the hub of the crate. It models a card as four real lines (the
//! `BEGIN` / `VERSION` envelope, the property lines, the `END`), made of the
//! nodes in the sibling modules ([`line`](crate::v40::tree::line),
//! [`param`](crate::v40::tree::param), [`value`](crate::v40::tree::value),
//! [`leaf`](crate::v40::tree::leaf)). It knows nothing about what a property *means*.
//! It is filled from bytes (`parse`) or from typed properties (`push`), exports
//! raw contents ([`Display`](core::fmt::Display) / `to_string`), and offers typed
//! access by lens (`prop`, `prop_mut`, `remove`). The semantic projection
//! ([`decode`](VcardCst::decode)) and the codec live in the
//! [`decode`](crate::v40::tree::decode) / [`encode`](crate::v40::tree::encode) siblings.

use core::fmt;

use alloc::{string::ToString, vec::Vec};

use crate::v40::{
    prop::VcardProp,
    tree::{error::VcardParseError, line::VcardLine, prop::VcardPropLens},
    vcard::{VCARD, VCARD_BEGIN, VCARD_END},
    version::{VCARD_VERSION, VCARD_VERSION_40},
};

/// A whole card as raw syntax: BEGIN, VERSION, the property lines, END. All four
/// are real lines so nothing is reconstructed by rule.
#[derive(Clone, Debug)]
pub struct VcardCst<'a> {
    /// The BEGIN line.
    pub begin: VcardLine<'a>,
    /// The VERSION line.
    pub version: VcardLine<'a>,
    /// The property lines, in source order.
    pub props: Vec<VcardLine<'a>>,
    /// The END line.
    pub end: VcardLine<'a>,
}

impl<'a> VcardCst<'a> {
    /// Start an empty vCard 4.0, BEGIN/VERSION/END seeded, ready for properties.
    pub fn v4() -> Self {
        Self {
            begin: VcardLine::text(VCARD_BEGIN, VCARD),
            version: VcardLine::text(VCARD_VERSION, VCARD_VERSION_40),
            props: Vec::new(),
            end: VcardLine::text(VCARD_END, VCARD),
        }
    }

    /// Parse exactly one card from raw text, borrowing it for the Cst lifetime.
    pub fn parse(input: &'a str) -> Result<Self, VcardParseError> {
        let (begin, mut rest) = VcardLine::take(input)?;
        if !begin.name.get().eq_ignore_ascii_case(VCARD_BEGIN) {
            return Err(VcardParseError::ExpectedBegin(begin.name.get().to_string()));
        }

        let (version, tail) = VcardLine::take(rest)?;
        rest = tail;
        if !version.name.get().eq_ignore_ascii_case(VCARD_VERSION) {
            let v = version.name.get().to_string();
            return Err(VcardParseError::ExpectedVersion(v));
        }

        let mut props = Vec::new();

        loop {
            if rest.is_empty() {
                return Err(VcardParseError::MissingEnd(input.to_string()));
            }

            let (line, tail) = VcardLine::take(rest)?;
            rest = tail;

            if line.name.get().eq_ignore_ascii_case(VCARD_END) {
                return Ok(Self {
                    begin,
                    version,
                    props,
                    end: line,
                });
            }

            props.push(line);
        }
    }

    // --- write: build / edit

    /// Append a typed property, encoding it into a line. Adding to a *parsed*
    /// card leaves every existing line byte for byte intact (they stay
    /// borrowed); only the new line is canonical. The building primitive.
    pub fn push(&mut self, prop: VcardProp<'a>) -> &mut Self {
        self.props.push(prop.encode());
        self
    }

    /// Remove every property of type `L`.
    pub fn remove<L: VcardPropLens>(&mut self) -> &mut Self {
        self.props
            .retain(|line| !line.name.get().eq_ignore_ascii_case(L::NAME));
        self
    }

    // --- read: typed access

    /// The first property of type `L`, decoded into a borrowed snapshot.
    pub fn prop<L: VcardPropLens>(&self) -> Option<L::Target<'_>> {
        self.props
            .iter()
            .find(|line| line.name.get().eq_ignore_ascii_case(L::NAME))
            .map(|line| L::decode(&line.value))
    }

    /// The first property of type `L`, as a typed cursor for in-place editing.
    pub fn prop_mut<L: VcardPropLens>(&mut self) -> Option<L::Cursor<'_, 'a>> {
        self.props
            .iter_mut()
            .find(|line| line.name.get().eq_ignore_ascii_case(L::NAME))
            .map(|line| L::cursor(line))
    }
}

impl fmt::Display for VcardCst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.begin, self.version)?;

        for prop in &self.props {
            write!(f, "{prop}")?;
        }

        write!(f, "{}", self.end)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

    use crate::v40::{
        param::VcardParam,
        prop::VcardProp,
        tree::{cst::VcardCst, prop::n::N},
        value::{VcardUnknownValue, VcardValue, n::VcardN, text::VcardText},
        vcard::Vcard,
        version::VcardVersion,
    };

    const CARD: &str = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "N;PID=1:Doe;John;;Dr.;\r\n",
        "FN:John Doe\r\n",
        "END:VCARD\r\n",
    );

    #[test]
    fn round_trips_byte_for_byte() {
        let card = VcardCst::parse(CARD).unwrap();
        assert_eq!(card.to_string(), CARD);
    }

    #[test]
    fn reads_a_property_through_its_lens() {
        let card = VcardCst::parse(CARD).unwrap();
        let name = card.prop::<N>().expect("an N property");

        assert_eq!(name.family, vec![Cow::Borrowed("Doe")]);
        assert_eq!(name.given, vec![Cow::Borrowed("John")]);
        assert_eq!(name.prefixes, vec![Cow::Borrowed("Dr.")]);
    }

    #[test]
    fn pushes_a_typed_property_onto_a_parsed_card() {
        let mut card = VcardCst::parse(CARD).unwrap();
        card.push(VcardProp::email(Vec::new(), "john@doe.example".into()));

        let out = card.to_string();
        // existing lines kept verbatim; only the appended one is canonical.
        assert!(out.contains("N;PID=1:Doe;John;;Dr.;\r\n"), "{out}");
        assert!(out.contains("EMAIL:john@doe.example\r\n"), "{out}");
    }

    #[test]
    fn removes_every_property_of_a_kind() {
        let mut card = VcardCst::parse(CARD).unwrap();
        card.remove::<N>();
        assert_eq!(
            card.to_string(),
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n",
        );
    }

    #[test]
    fn builds_a_card_from_decoded_types() {
        let card = Vcard {
            version: VcardVersion::V40,
            properties: vec![VcardProp {
                name: Cow::Borrowed("N"),
                params: Vec::new(),
                value: VcardValue::N(VcardN {
                    family: vec![Cow::Borrowed("Doe")],
                    given: vec![Cow::Borrowed("John")],
                    additional: Vec::new(),
                    prefixes: vec![Cow::Borrowed("Dr.")],
                    suffixes: Vec::new(),
                }),
            }],
        };

        assert_eq!(
            card.to_string(),
            "BEGIN:VCARD\r\nVERSION:4.0\r\nN:Doe;John;;Dr.;\r\nEND:VCARD\r\n",
        );
    }

    #[test]
    fn decodes_the_whole_card() {
        let cst = VcardCst::parse(CARD).unwrap();
        let vcard = cst.decode();

        assert_eq!(vcard.version, VcardVersion::V40);
        assert_eq!(vcard.properties.len(), 2);

        let n = &vcard.properties[0];
        assert_eq!(n.name, "N");
        assert_eq!(n.params, vec![VcardParam::Pid(vec![Cow::Borrowed("1")])]);
        assert!(matches!(n.value, VcardValue::N(_)));

        let fnn = &vcard.properties[1];
        assert_eq!(fnn.name, "FN");
        assert_eq!(
            fnn.value,
            VcardValue::Text(VcardText(Cow::Borrowed("John Doe"))),
        );

        // the canonical rebuild reproduces the (already-canonical) card.
        assert_eq!(vcard.to_string(), CARD);
    }

    #[test]
    fn keeps_an_unknown_property_round_tripping() {
        let card = "BEGIN:VCARD\r\nVERSION:4.0\r\nX-CUSTOM:a;b,c\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(card).unwrap();
        let vcard = cst.decode();

        match &vcard.properties[0].value {
            VcardValue::Unknown(VcardUnknownValue { components }) => {
                assert_eq!(components.len(), 2);
                assert_eq!(components[1], vec![Cow::Borrowed("b"), Cow::Borrowed("c")]);
            }
            other => panic!("expected Unknown, got {other:?}"),
        }
        assert_eq!(vcard.to_string(), card);
    }

    #[test]
    fn unfolds_folded_lines_across_the_card() {
        let folded = "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:a long\r\n  note\r\nEND:VCARD\r\n";
        let card = VcardCst::parse(folded).unwrap();

        // the folded value is unfolded on parse, and serialized unfolded.
        assert_eq!(
            card.to_string(),
            "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:a long note\r\nEND:VCARD\r\n",
        );
        // re-parsing the output is then byte-stable (a fixpoint).
        let output = card.to_string();
        let reparsed = VcardCst::parse(&output).unwrap();
        assert_eq!(reparsed.to_string(), output);
    }

    #[test]
    fn tolerates_blank_lines_and_a_missing_final_break() {
        // a stray blank line after VERSION, and no trailing break after END.
        let input = "BEGIN:VCARD\r\nVERSION:4.0\r\n\r\nFN:John\r\nEND:VCARD";
        let card = VcardCst::parse(input).unwrap();

        assert_eq!(card.decode().properties.len(), 1);
        // the blank line is dropped; the missing final break is preserved.
        assert_eq!(
            card.to_string(),
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John\r\nEND:VCARD",
        );
    }
}
