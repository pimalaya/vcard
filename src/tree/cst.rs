//! # Concrete syntax tree
//!
//! The core representation: a whole vCard as generic, byte-faithful syntax.
//!
//! [`VcardCst`] is the hub of the crate. It models a card as four real lines
//! (the `BEGIN` / `VERSION` envelope, the property lines, the `END`), made of
//! the nodes in the sibling modules ([`line`](crate::tree::line),
//! [`param`](crate::tree::param), [`value`](crate::tree::value),
//! [`leaf`](crate::tree::leaf)). It knows nothing about what a property
//! *means*. It is filled from bytes (`parse`) or from typed properties
//! (`push`), exports raw contents ([`Display`](core::fmt::Display) /
//! `to_string`), and offers typed access by lens (`prop`, `prop_mut`,
//! `remove`). The semantic projection ([`decode`](VcardCst::decode)) and the
//! codec live in the [`decode`](crate::tree::codec::decode) /
//! [`encode`](crate::tree::codec::encode) siblings.
//!
//! # Examples
//!
//! Parse raw bytes into a CST, edit a field in place (byte-preservingly), then
//! project onto the decoded model:
//!
//! ```rust
//! use vcard::tree::cst::VcardCst;
//! use vcard::tree::prop::r#fn::FN;
//! use vcard::version::VcardVersion;
//!
//! // 1. Parse raw bytes into the byte-faithful syntax tree (round-trips exactly).
//! let raw = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n";
//! let mut cst = VcardCst::parse(raw).unwrap();
//! assert_eq!(cst.to_string(), raw);
//!
//! // 2. Edit one field through its lens; every untouched byte is preserved.
//! cst.prop_mut::<FN>().unwrap().set_text("Jane Doe");
//! assert_eq!(
//!     cst.to_string(),
//!     "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\nEND:VCARD\r\n",
//! );
//!
//! // 3. Project onto the decoded, version-agnostic model.
//! let card = cst.decode();
//! assert_eq!(card.version, VcardVersion::V4_0);
//! assert_eq!(&*card.properties[0].name, "FN");
//! ```
//!
//! Build a property from the strict builder, validate a whole card into a
//! [`Valid`](crate::tree::vcard::validate::Valid) proof, then turn it back into a
//! CST:
//!
//! ```rust
//! use vcard::tree::cst::VcardCst;
//! use vcard::tree::vcard::builder::VcardPropBuilder;
//! use vcard::tree::prop::r#fn::FN;
//! use vcard::vcard::Vcard;
//! use vcard::value::VcardValue;
//! use vcard::value::text::VcardText;
//! use vcard::version::VcardVersion;
//! use std::borrow::Cow;
//!
//! // 1. Build a property strictly against its spec.
//! let fn_prop = VcardPropBuilder::<FN>::new(VcardVersion::V4_0)
//!     .build(VcardValue::Text(VcardText(Cow::Borrowed("John Doe"))))
//!     .unwrap();
//!
//! // 2. Assemble a card and validate it into a proof.
//! let card = Vcard {
//!     version: VcardVersion::V4_0,
//!     properties: vec![fn_prop],
//! };
//! let valid = card.validate().expect("a conformant 4.0 card");
//!
//! // 3. A Valid<Vcard> converts back into a byte tree for free.
//! let cst = VcardCst::from(valid);
//! assert!(cst.to_string().contains("FN:John Doe\r\n"));
//! ```

use core::fmt;

use alloc::{string::ToString, vec, vec::Vec};

use crate::{
    prop::VcardProp,
    tree::{codec::mode::Escaper, error::VcardParseError, line::VcardLine, prop::VcardPropLens},
    version::VcardVersion,
};

/// A whole card as raw syntax: a `BEGIN` line, the property lines (the
/// `VERSION` line among them, wherever it falls), and an `END` line. All are
/// real lines so nothing is reconstructed by rule, and `VERSION` keeps its
/// source position.
#[derive(Clone, Debug)]
pub struct VcardCst<'a> {
    /// The BEGIN line.
    pub begin: VcardLine<'a>,
    /// The property lines, in source order, including the `VERSION` line.
    pub props: Vec<VcardLine<'a>>,
    /// The END line.
    pub end: VcardLine<'a>,
}

impl<'a> VcardCst<'a> {
    /// Start an empty vCard 4.0, BEGIN/VERSION/END seeded, ready for
    /// properties.
    pub fn v4() -> Self {
        Self {
            begin: VcardLine::text("BEGIN", "VCARD"),
            props: vec![VcardLine::text("VERSION", &*VcardVersion::V4_0)],
            end: VcardLine::text("END", "VCARD"),
        }
    }

    /// Parse the first card from raw text, borrowing it for the Cst lifetime.
    /// `VERSION` is taken as an ordinary property wherever it appears (or not
    /// at all): the parser is liberal about its position, the way real cards
    /// are. Any input after the card's `END` line is ignored; use
    /// [`parse_many`](Self::parse_many) to read every card.
    pub fn parse(input: &'a str) -> Result<Self, VcardParseError> {
        Self::take_card(input).map(|(card, _rest)| card)
    }

    /// Parse every card in the input, lazily, one item per card (or the parse
    /// error that stopped iteration). Blank lines between cards are skipped. For
    /// multi-card `.vcf` files and CardDAV address books.
    ///
    /// ```rust
    /// use vcard::tree::cst::VcardCst;
    ///
    /// let file = concat!(
    ///     "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Alice\r\nEND:VCARD\r\n",
    ///     "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Bob\r\nEND:VCARD\r\n",
    /// );
    /// let cards: Result<Vec<_>, _> = VcardCst::parse_many(file).collect();
    /// assert_eq!(cards.unwrap().len(), 2);
    /// ```
    pub fn parse_many(input: &'a str) -> impl Iterator<Item = Result<Self, VcardParseError>> {
        let mut rest = input;

        core::iter::from_fn(move || {
            rest = rest.trim_start_matches(['\r', '\n']);
            if rest.is_empty() {
                return None;
            }

            match Self::take_card(rest) {
                Ok((card, tail)) => {
                    rest = tail;
                    Some(Ok(card))
                }
                Err(error) => {
                    // NOTE: stop after the first failure; a malformed card
                    // leaves no reliable boundary to resume from.
                    rest = "";
                    Some(Err(error))
                }
            }
        })
    }

    /// Take one card off the front of `input`, returning it and the unconsumed
    /// rest. The card's `END` is the one at `BEGIN`/`END` depth zero, so a 2.1
    /// inline `AGENT` (a whole nested `BEGIN`..`END`) does not close the card
    /// early; the nested lines are kept verbatim so the card round-trips.
    fn take_card(input: &'a str) -> Result<(Self, &'a str), VcardParseError> {
        let (begin, mut rest) = VcardLine::take(input)?;

        if !begin.name.get().eq_ignore_ascii_case("BEGIN") {
            return Err(VcardParseError::ExpectedBegin(begin.name.get().to_string()));
        }

        let mut props: Vec<VcardLine<'a>> = Vec::new();
        let mut depth = 0usize;

        loop {
            if rest.is_empty() {
                return Err(VcardParseError::MissingEnd(input.to_string()));
            }

            let (line, tail) = VcardLine::take(rest)?;
            rest = tail;

            let name = line.name.get();

            if name.eq_ignore_ascii_case("END") {
                if let Some(next) = depth.checked_sub(1) {
                    // NOTE: a nested END closes an embedded (AGENT) card, not
                    // this one; keep it as a verbatim line.
                    depth = next;
                    props.push(line);
                    continue;
                }

                // NOTE: VERSION can sit anywhere, so the escaping mode is only
                // known once the whole card is parsed: stamp every value node
                // with it.
                let escaper = props
                    .iter()
                    .find(|line| line.name.get().eq_ignore_ascii_case("VERSION"))
                    .map(|line| Escaper::for_version_str(line.raw_value()))
                    .unwrap_or_default();

                for line in &mut props {
                    line.value.escaper = escaper;
                }

                return Ok((
                    Self {
                        begin,
                        props,
                        end: line,
                    },
                    rest,
                ));
            }

            if name.eq_ignore_ascii_case("BEGIN") {
                depth += 1;
            }

            props.push(line);
        }
    }

    /// The `VERSION` line, wherever it sits among the properties (or `None`).
    pub fn version_line(&self) -> Option<&VcardLine<'a>> {
        self.props
            .iter()
            .find(|line| line.name.get().eq_ignore_ascii_case("VERSION"))
    }

    /// The card's version indicator, read from its `VERSION` line. An
    /// unrecognised or missing version normalises to
    /// [`V4_0`](VcardVersion::V4_0).
    pub fn version(&self) -> VcardVersion {
        self.version_line()
            .and_then(|line| line.raw_value().parse().ok())
            .unwrap_or(VcardVersion::V4_0)
    }

    /// Append a typed property, encoding it into a line. Adding to a *parsed*
    /// card leaves every existing line byte for byte intact (they stay
    /// borrowed); only the new line is canonical. The building primitive.
    pub fn push(&mut self, prop: VcardProp<'a>) -> &mut Self {
        let escaper = self
            .version_line()
            .map(|line| Escaper::for_version_str(line.raw_value()))
            .unwrap_or_default();

        self.props.push(prop.encode(escaper));
        self
    }

    /// Remove every property of type `L`.
    pub fn remove<L: VcardPropLens>(&mut self) -> &mut Self {
        self.props
            .retain(|line| !line.name.get().eq_ignore_ascii_case(&L::KIND));
        self
    }

    /// The first property of type `L`, decoded into a borrowed snapshot. The
    /// card version is threaded through so version-specific value shapes
    /// (`GEO`, the binary props) decode the same way the whole-card
    /// [`decode`](Self::decode) does.
    pub fn prop<L: VcardPropLens>(&self) -> Option<L::Target<'_>> {
        let version = self.version();
        self.props
            .iter()
            .find(|line| line.name.get().eq_ignore_ascii_case(&L::KIND))
            .map(|line| L::decode(line, version))
    }

    /// The first property of type `L`, as a typed cursor for in-place editing.
    pub fn prop_mut<L: VcardPropLens>(&mut self) -> Option<L::Cursor<'_, 'a>> {
        self.props
            .iter_mut()
            .find(|line| line.name.get().eq_ignore_ascii_case(&L::KIND))
            .map(|line| L::cursor(line))
    }
}

impl fmt::Display for VcardCst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.begin)?;

        for prop in &self.props {
            write!(f, "{prop}")?;
        }

        write!(f, "{}", self.end)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

    use crate::prop::VcardPropKind;
    use crate::version::VcardVersion;
    use crate::{
        param::VcardParam,
        prop::VcardProp,
        tree::{cst::VcardCst, prop::n::N},
        value::{VcardUnknownValue, VcardValue, n::VcardN, text::VcardText},
        vcard::Vcard,
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
    fn parses_many_cards_from_one_input() {
        let a = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nEND:VCARD\r\n";
        let b = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:B\r\nEND:VCARD\r\n";
        // Two cards separated by a blank line, which the parser skips.
        let input = concat!(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nEND:VCARD\r\n",
            "\r\n",
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:B\r\nEND:VCARD\r\n",
        );

        let cards = VcardCst::parse_many(input)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].to_string(), a);
        assert_eq!(cards[1].to_string(), b);
    }

    #[test]
    fn keeps_a_nested_agent_card_intact() {
        // A 2.1 inline AGENT embeds a whole vCard; its inner END must not close
        // the outer card, and the whole thing round-trips byte for byte.
        let raw = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:2.1\r\n",
            "FN:Has Agent\r\n",
            "AGENT:\r\n",
            "BEGIN:VCARD\r\n",
            "VERSION:2.1\r\n",
            "N:Friday;Fred\r\n",
            "TEL;WORK;VOICE:+1-213-555-1234\r\n",
            "END:VCARD\r\n",
            "END:VCARD\r\n",
        );
        assert_eq!(VcardCst::parse(raw).unwrap().to_string(), raw);
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
        card.push(VcardProp {
            name: VcardPropKind::Email.into(),
            params: [].into(),
            value: VcardValue::Text("john@doe.example".into()),
        });

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
            version: VcardVersion::V4_0,
            properties: vec![VcardProp {
                name: "N".into(),
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
    fn encodes_a_built_card_with_version_specific_escaping() {
        let note = |version| Vcard {
            version,
            properties: vec![VcardProp {
                name: "NOTE".into(),
                params: Vec::new(),
                value: VcardValue::Text(VcardText(Cow::Borrowed("a,b;c"))),
            }],
        };

        // 2.1 escapes only `;`, leaving `,` literal.
        assert_eq!(
            note(VcardVersion::V2_1).to_string(),
            "BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE:a,b\\;c\r\nEND:VCARD\r\n",
        );
        // 4.0 escapes both `,` and `;` (modern rules).
        assert_eq!(
            note(VcardVersion::V4_0).to_string(),
            "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:a\\,b\\;c\r\nEND:VCARD\r\n",
        );

        // Pushing onto a parsed 2.1 card uses the card's 2.1 escaping.
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nFN:X\r\nEND:VCARD\r\n").unwrap();
        card.push(VcardProp {
            name: "NOTE".into(),
            params: Vec::new(),
            value: VcardValue::Text(VcardText(Cow::Borrowed("a,b;c"))),
        });
        assert!(
            card.to_string().contains("NOTE:a,b\\;c\r\n"),
            "{}",
            card.to_string(),
        );
    }

    #[test]
    fn decodes_the_whole_card() {
        let cst = VcardCst::parse(CARD).unwrap();
        let vcard = cst.decode();

        assert_eq!(vcard.version, VcardVersion::V4_0);
        assert_eq!(vcard.properties.len(), 2);

        let n = &vcard.properties[0];
        assert_eq!(&*n.name, "N");
        assert_eq!(n.params, vec![VcardParam::Pid(vec![Cow::Borrowed("1")])]);
        assert!(matches!(n.value, VcardValue::N(_)));

        let fnn = &vcard.properties[1];
        assert_eq!(&*fnn.name, "FN");
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
    fn reads_and_edits_a_legacy_property_through_its_lens() {
        use crate::tree::prop::label::LABEL;

        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:3.0\r\nLABEL:Old\r\nFN:X\r\nEND:VCARD\r\n")
                .unwrap();

        assert_eq!(
            card.prop::<LABEL>().unwrap(),
            VcardText(Cow::Borrowed("Old"))
        );

        card.prop_mut::<LABEL>().unwrap().set_text("New");
        assert_eq!(
            card.to_string(),
            "BEGIN:VCARD\r\nVERSION:3.0\r\nLABEL:New\r\nFN:X\r\nEND:VCARD\r\n",
        );
    }

    #[test]
    fn parses_version_anywhere_keeping_its_position() {
        // VERSION on the third line, as RFC 2426 example cards do.
        let card = "BEGIN:VCARD\r\nBDAY:1980-01-02\r\nVERSION:3.0\r\nFN:X\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(card).unwrap();

        // byte-faithful: VERSION keeps its source position.
        assert_eq!(cst.to_string(), card);

        let vcard = cst.decode();
        assert_eq!(vcard.version, VcardVersion::V3_0);
        // VERSION is the indicator, not a property: only BDAY and FN remain.
        assert_eq!(vcard.properties.len(), 2);
    }

    #[test]
    fn parses_a_card_with_no_version() {
        let card = "BEGIN:VCARD\r\nFN:X\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(card).unwrap();

        // The raw card round-trips byte for byte (no VERSION line invented)...
        assert_eq!(cst.to_string(), card);
        // ...but the decoded model normalises a missing version to 4.0.
        assert_eq!(cst.decode().version, VcardVersion::V4_0);
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
