//! # Concrete syntax tree
//!
//! The core representation: a whole vCard as generic, byte-faithful syntax.
//!
//! [`VcardCst`] is the hub of the crate. It models a card as its real lines: an
//! optional `BEGIN` / `END` envelope (absent for a bare RFC 2425 record) and
//! the property lines between them (the `VERSION` line among them), made of the
//! nodes in the sibling modules ([`line`](crate::tree::line),
//! [`param`](crate::tree::param), [`value`](crate::tree::value),
//! [`leaf`](crate::tree::leaf)). It knows nothing about what a property
//! *means*. It is filled from bytes (`parse`) or from typed properties
//! (`push`), exports its bytes byte-faithfully
//! ([`to_bytes`](VcardCst::to_bytes), or the lossy-for-non-UTF-8
//! [`Display`](core::fmt::Display) / `to_string`), and offers typed access by
//! lens (`prop`, `prop_mut`, `remove`). The semantic projection
//! ([`decode`](VcardCst::decode)) and the codec live in the
//! [`decode`](crate::tree::codec::decode) /
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
//! // 1. Parse raw bytes into the byte-faithful syntax tree (round-trips
//! // exactly).
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
//! [`VcardValid`](crate::tree::vcard::validate::VcardValid) proof, then turn it
//! back into a CST:
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
//! // 3. A VcardValid<Vcard> converts back into a byte tree for free.
//! let cst = VcardCst::from(valid);
//! assert!(cst.to_string().contains("FN:John Doe\r\n"));
//! ```

use core::fmt;

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{
    prop::{VcardProp, VcardPropKind},
    tree::{
        codec::mode::VcardEscaper,
        error::VcardParseError,
        line::VcardLine,
        prop::{cardinality::VcardPropCardinality, lens::VcardPropLens, spec::prop_spec},
    },
    value::VcardValue,
    version::VcardVersion,
};

/// A whole card as raw syntax: an optional `BEGIN` / `END` envelope and the
/// property lines between them (the `VERSION` line among them, wherever it
/// falls). All are real lines so nothing is reconstructed by rule, `VERSION`
/// keeps its source position, and the envelope is absent only for a bare RFC
/// 2425 directory record.
#[derive(Clone, Debug)]
pub struct VcardCst<'a> {
    /// The BEGIN line, or `None` for a bare RFC 2425 directory record parsed
    /// without a `BEGIN:VCARD` envelope.
    pub begin: Option<VcardLine<'a>>,
    /// The property lines, in source order, including the `VERSION` line.
    pub props: Vec<VcardLine<'a>>,
    /// The END line, absent exactly when [`begin`](Self::begin) is.
    pub end: Option<VcardLine<'a>>,
}

impl<'a> VcardCst<'a> {
    /// Start an empty vCard 4.0, BEGIN/VERSION/END seeded, ready for
    /// properties.
    pub fn v4() -> Self {
        Self {
            begin: Some(VcardLine::text("BEGIN", "VCARD")),
            props: vec![VcardLine::text("VERSION", &*VcardVersion::V4_0)],
            end: Some(VcardLine::text("END", "VCARD")),
        }
    }

    /// Parse the first card from raw text, borrowing it for the Cst lifetime.
    /// `VERSION` is an ordinary property wherever it appears, or nowhere: the
    /// parser is as liberal about its position as real cards are. Anything
    /// after `END` is ignored; use [`parse_many`](Self::parse_many) for a file.
    ///
    /// A bare RFC 2425 record with no `BEGIN:VCARD` is also accepted: every
    /// line becomes a property, [`begin`](Self::begin) / [`end`](Self::end)
    /// stay `None`, and it round-trips without a synthesised envelope. Only
    /// here, since without the envelope `parse_many` has no record boundary.
    pub fn parse<T: AsRef<[u8]> + ?Sized>(input: &'a T) -> Result<Self, VcardParseError> {
        // NOTE: Trim leading blank-line bytes before the bare-vs-wrapped
        // decision, so it does not hinge on stray leading whitespace:
        // serialization normalises that away, so a classification that depended
        // on it would make the round-trip non-idempotent (a bare record whose
        // first line then reads as BEGIN would reparse as a wrapped card).
        let input = trim_leading_eol(input.as_ref());
        let (first, _rest) = VcardLine::take(input)?;

        if first.name.get().eq_ignore_ascii_case("BEGIN") {
            Self::take_card(input).map(|(card, _rest)| card)
        } else {
            Self::parse_bare(input)
        }
    }

    /// Parse a bare directory record: every line is a property, with no
    /// envelope. The escaping mode is stamped from a `VERSION` line if the
    /// record happens to carry one, else the default.
    fn parse_bare(input: &'a [u8]) -> Result<Self, VcardParseError> {
        let mut props: Vec<VcardLine<'a>> = Vec::new();
        let mut rest = trim_leading_eol(input);

        while !rest.is_empty() {
            let (line, tail) = VcardLine::take(rest)?;
            props.push(line);
            rest = trim_leading_eol(tail);
        }

        let escaper = props
            .iter()
            .find(|line| line.name.get().eq_ignore_ascii_case("VERSION"))
            .map(|line| VcardEscaper::for_version_str(line.raw_value_str().as_ref()))
            .unwrap_or_default();

        for line in &mut props {
            line.value.escaper = escaper;
        }

        Ok(Self {
            begin: None,
            props,
            end: None,
        })
    }

    /// Parse every card in the input, lazily, one item per card (or the parse
    /// error that stopped iteration). Blank lines between cards are skipped.
    /// For multi-card `.vcf` files and CardDAV address books.
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
    pub fn parse_many<T: AsRef<[u8]> + ?Sized>(
        input: &'a T,
    ) -> impl Iterator<Item = Result<Self, VcardParseError>> {
        let mut rest = input.as_ref();

        core::iter::from_fn(move || {
            rest = trim_leading_eol(rest);
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
                    rest = b"";
                    Some(Err(error))
                }
            }
        })
    }

    /// Take one card off the front of `input`, returning it and the unconsumed
    /// rest. The card's `END` is the one at `BEGIN`/`END` depth zero, so a 2.1
    /// inline `AGENT` (a whole nested `BEGIN`..`END`) does not close the card
    /// early; the nested lines are kept verbatim so the card round-trips.
    fn take_card(input: &'a [u8]) -> Result<(Self, &'a [u8]), VcardParseError> {
        let (begin, mut rest) = VcardLine::take(input)?;

        if !begin.name.get().eq_ignore_ascii_case("BEGIN") {
            return Err(VcardParseError::ExpectedBegin(begin.name.get().to_string()));
        }

        let mut props: Vec<VcardLine<'a>> = Vec::new();
        let mut depth = 0usize;

        loop {
            if rest.is_empty() {
                return Err(VcardParseError::MissingEnd(
                    String::from_utf8_lossy(input).into_owned(),
                ));
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
                    .map(|line| VcardEscaper::for_version_str(line.raw_value_str().as_ref()))
                    .unwrap_or_default();

                for line in &mut props {
                    line.value.escaper = escaper;
                }

                return Ok((
                    Self {
                        begin: Some(begin),
                        props,
                        end: Some(line),
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
            .and_then(|line| line.raw_value_str().parse().ok())
            .unwrap_or(VcardVersion::V4_0)
    }

    /// Append a typed property, encoding it into a line. Adding to a *parsed*
    /// card leaves every existing line byte for byte intact (they stay
    /// borrowed); only the new line is canonical. The building primitive.
    pub fn push(&mut self, prop: VcardProp<'a>) -> &mut Self {
        let escaper = self
            .version_line()
            .map(|line| VcardEscaper::for_version_str(line.raw_value_str().as_ref()))
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

    /// Append a blank instance of every required property the card is missing
    /// for its version: in practice `N` in 2.1 / 3.0 and `FN` from 3.0 on.
    /// Strict servers (iCloud, Fastmail) reject a 3.0 card with no `N`, and an
    /// empty `N:;;;;` satisfies them without inventing a display value.
    ///
    /// The repair half of
    /// [`Vcard::validate`](crate::vcard::Vcard::validate)'s cardinality check.
    /// Idempotent, and existing lines stay byte for byte intact (see
    /// [`push`](Self::push)).
    pub fn fill_required(&mut self) -> &mut Self {
        let version = self.version();
        for kind in VcardPropKind::ALL {
            let spec = prop_spec(kind);
            if !(spec.allowed_versions)().contains(&version) {
                continue;
            }
            let required = matches!(
                (spec.cardinality)(version),
                VcardPropCardinality::ExactlyOne | VcardPropCardinality::OneOrMore,
            );
            let present = self
                .props
                .iter()
                .any(|line| line.name.get().eq_ignore_ascii_case(&kind));
            if !required || present {
                continue;
            }
            if let Some(value_kind) = (spec.allowed_values)(version).first() {
                self.push(VcardProp {
                    name: kind.into(),
                    params: Vec::new(),
                    value: VcardValue::empty(*value_kind),
                });
            }
        }
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

    /// Own every borrowed leaf, detaching the card from the source bytes so it
    /// can outlive them.
    pub fn into_static(self) -> VcardCst<'static> {
        VcardCst {
            begin: self.begin.map(VcardLine::into_static),
            props: self.props.into_iter().map(VcardLine::into_static).collect(),
            end: self.end.map(VcardLine::into_static),
        }
    }

    /// Serialize the card to raw bytes, exactly as parsed. Unlike
    /// [`Display`](core::fmt::Display) (which is lossy for a value carrying a
    /// non-UTF-8 charset), this is always byte-faithful.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        if let Some(begin) = &self.begin {
            begin.write_bytes(&mut out);
        }
        for prop in &self.props {
            prop.write_bytes(&mut out);
        }
        if let Some(end) = &self.end {
            end.write_bytes(&mut out);
        }

        out
    }
}

/// Skip leading blank-line bytes (`\r` / `\n`) between cards.
fn trim_leading_eol(mut bytes: &[u8]) -> &[u8] {
    while let Some((first, rest)) = bytes.split_first() {
        if matches!(first, b'\r' | b'\n') {
            bytes = rest;
        } else {
            break;
        }
    }

    bytes
}

impl fmt::Display for VcardCst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(begin) = &self.begin {
            write!(f, "{begin}")?;
        }

        for prop in &self.props {
            write!(f, "{prop}")?;
        }

        if let Some(end) = &self.end {
            write!(f, "{end}")?;
        }

        Ok(())
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
        value::{VcardValue, VcardValueUnknown, n::VcardN, text::VcardText},
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
    fn round_trips_a_non_utf8_value_byte_for_byte() {
        use crate::tree::prop::note::NOTE;

        // NOTE: A vCard 2.1 NOTE in ISO-8859-1: the 0xE9 byte ('é' in Latin-1)
        // is not valid UTF-8, so this card could not be a &str at all.
        let mut raw = Vec::new();
        raw.extend_from_slice(b"BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE;CHARSET=ISO-8859-1:caf");
        raw.push(0xE9);
        raw.extend_from_slice(b"\r\nEND:VCARD\r\n");

        let cst = VcardCst::parse(&raw).unwrap();

        // NOTE: to_bytes() is byte-faithful, including the raw 0xE9 octet.
        assert_eq!(cst.to_bytes(), raw);

        // NOTE: CHARSET rides along as a plain parameter; the value bytes are
        // reachable raw through the lens cursor, not transcoded.
        let mut cst = cst;
        assert_eq!(
            cst.prop_mut::<NOTE>().unwrap().bytes().as_ref(),
            &[b'c', b'a', b'f', 0xE9],
        );
        assert!(
            cst.decode().properties[0]
                .params
                .contains(&VcardParam::Charset(Cow::Borrowed("ISO-8859-1"))),
        );
    }

    #[test]
    fn rejects_a_non_utf8_property_name() {
        use crate::tree::error::VcardParseError;

        // NOTE: Only a value may carry a foreign charset; a non-UTF-8 name or
        // parameter is a hard parse error.
        let mut raw = Vec::new();
        raw.extend_from_slice(b"BEGIN:VCARD\r\nVERSION:4.0\r\nX-");
        raw.push(0xFF);
        raw.extend_from_slice(b":v\r\nEND:VCARD\r\n");

        assert!(matches!(
            VcardCst::parse(&raw),
            Err(VcardParseError::NonUtf8Header(_)),
        ));
    }

    #[test]
    fn round_trips_fuzz_regressions() {
        // NOTE: Inputs the coverage-guided fuzzer found where a parsed card
        // serialized to bytes that either failed to reparse or reparsed to
        // different bytes. Every one must now parse, and its serialization must
        // be a byte-stable fixpoint (its own output reparses identically).
        let cases: &[&[u8]] = &[
            // NOTE: QP value left ending in `=` (a dangling soft-break marker)
            // would, once serialized, re-trigger soft-break joining and swallow
            // the END line on reparse. Two ways in: a soft-break join over a
            // blank continuation (`x==` -> `x=`)...
            b"BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE;ENCODING=QUOTED-PRINTABLE:x==\r\n\r\nEND:VCARD\r\n",
            // NOTE: ...and a folded continuation that appends the `=` (`Luo` +
            // ` =`).
            b"BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE;ENCODING=QUOTED-PRINTABLE:Luo\r\n =\r\nEND:VCARD\r\n",
            // NOTE: A dropped blank line before a space-prefixed (fold-looking)
            // line must not let that line fold into the previous one on
            // reparse.
            b"BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:a\r\n\r\n FN:b\r\nEND:VCARD\r\n",
            // NOTE: A line whose content starts with folding whitespace or a
            // stray CR must strip to a stable form (leading `\t`/`\r` here).
            b"\t\r:",
        ];

        for raw in cases {
            let cst = VcardCst::parse(raw).unwrap_or_else(|e| panic!("parse {raw:?}: {e}"));
            let bytes = cst.to_bytes();

            let reparsed =
                VcardCst::parse(&bytes).unwrap_or_else(|e| panic!("reparse {raw:?}: {e}"));
            assert_eq!(reparsed.to_bytes(), bytes, "not idempotent: {raw:?}");
        }
    }

    #[test]
    fn parses_a_bare_directory_record_without_an_envelope() {
        // NOTE: An RFC 2425 record with no BEGIN:VCARD wrapper: every line is a
        // property and no envelope is synthesised, so it round-trips exactly.
        let raw = "cn:Babs Jensen\r\nemail:babs@umich.edu\r\n";
        let cst = VcardCst::parse(raw).unwrap();

        assert!(cst.begin.is_none());
        assert!(cst.end.is_none());
        assert_eq!(cst.props.len(), 2);
        assert_eq!(cst.to_string(), raw);
    }

    #[test]
    fn parse_many_refuses_a_bare_record() {
        use crate::tree::error::VcardParseError;

        // NOTE: Without an envelope there is no reliable boundary between
        // records, so the multi-card reader rejects a bare record rather than
        // guess.
        let raw = "cn:Babs Jensen\r\nemail:babs@umich.edu\r\n";
        let first = VcardCst::parse_many(raw).next().unwrap();

        assert!(matches!(first, Err(VcardParseError::ExpectedBegin(_))));
    }

    #[test]
    fn parses_many_cards_from_one_input() {
        let a = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nEND:VCARD\r\n";
        let b = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:B\r\nEND:VCARD\r\n";
        // NOTE: Two cards separated by a blank line, which the parser skips.
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
        // NOTE: A 2.1 inline AGENT embeds a whole vCard; its inner END must not
        // close the outer card, and the whole thing round-trips byte for byte.
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
        // NOTE: existing lines kept verbatim; only the appended one is
        // canonical.
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
    fn fill_required_injects_the_mandatory_empty_n_into_a_3_0_card() {
        // NOTE: A 3.0 card with only a display name: FN but no N (mandatory).
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:3.0\r\nUID:x\r\nFN:Only\r\nEND:VCARD\r\n")
                .unwrap();
        card.fill_required();

        let out = card.to_string();
        assert!(out.contains("N:;;;;\r\n"), "{out}");
        // NOTE: Existing lines stay verbatim.
        assert!(out.contains("UID:x\r\n"), "{out}");
        assert!(out.contains("FN:Only\r\n"), "{out}");
        // NOTE: The repair makes the card conformant.
        assert!(card.decode().validate().is_ok());
    }

    #[test]
    fn fill_required_is_idempotent_and_leaves_valid_cards_untouched() {
        // NOTE: Running twice adds only the one missing N.
        let mut once =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Only\r\nEND:VCARD\r\n").unwrap();
        once.fill_required().fill_required();
        assert_eq!(once.to_string().matches("N:;;;;").count(), 1);

        // NOTE: 4.0 makes N optional, so a card with FN is left exactly as it
        // was.
        let mut v4 =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Only\r\nEND:VCARD\r\n").unwrap();
        let before = v4.to_string();
        assert_eq!(v4.fill_required().to_string(), before);

        // NOTE: A 3.0 card that already has N (and FN) is untouched too.
        let mut has = VcardCst::parse(
            "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Doe;Jane;;;\r\nFN:Jane\r\nEND:VCARD\r\n",
        )
        .unwrap();
        let before = has.to_string();
        assert_eq!(has.fill_required().to_string(), before);
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

        // NOTE: 2.1 escapes only `;`, leaving `,` literal.
        assert_eq!(
            note(VcardVersion::V2_1).to_string(),
            "BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE:a,b\\;c\r\nEND:VCARD\r\n",
        );
        // NOTE: 4.0 escapes both `,` and `;` (modern rules).
        assert_eq!(
            note(VcardVersion::V4_0).to_string(),
            "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:a\\,b\\;c\r\nEND:VCARD\r\n",
        );

        // NOTE: Pushing onto a parsed 2.1 card uses the card's 2.1 escaping.
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

        // NOTE: the canonical rebuild reproduces the (already-canonical) card.
        assert_eq!(vcard.to_string(), CARD);
    }

    #[test]
    fn keeps_an_unknown_property_round_tripping() {
        let card = "BEGIN:VCARD\r\nVERSION:4.0\r\nX-CUSTOM:a;b,c\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(card).unwrap();
        let vcard = cst.decode();

        match &vcard.properties[0].value {
            VcardValue::Unknown(VcardValueUnknown { components }) => {
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
        // NOTE: VERSION on the third line, as RFC 2426 example cards do.
        let card = "BEGIN:VCARD\r\nBDAY:1980-01-02\r\nVERSION:3.0\r\nFN:X\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(card).unwrap();

        // NOTE: byte-faithful: VERSION keeps its source position.
        assert_eq!(cst.to_string(), card);

        let vcard = cst.decode();
        assert_eq!(vcard.version, VcardVersion::V3_0);
        // NOTE: VERSION is the indicator, not a property: only BDAY and FN
        // remain.
        assert_eq!(vcard.properties.len(), 2);
    }

    #[test]
    fn parses_a_card_with_no_version() {
        let card = "BEGIN:VCARD\r\nFN:X\r\nEND:VCARD\r\n";
        let cst = VcardCst::parse(card).unwrap();

        // NOTE: The raw card round-trips byte for byte (no VERSION line
        // invented)...
        assert_eq!(cst.to_string(), card);
        // NOTE: ...but the decoded model normalises a missing version to 4.0.
        assert_eq!(cst.decode().version, VcardVersion::V4_0);
    }

    #[test]
    fn unfolds_folded_lines_across_the_card() {
        let folded = "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:a long\r\n  note\r\nEND:VCARD\r\n";
        let card = VcardCst::parse(folded).unwrap();

        // NOTE: the folded value is unfolded on parse, and serialized unfolded.
        assert_eq!(
            card.to_string(),
            "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:a long note\r\nEND:VCARD\r\n",
        );
        // NOTE: re-parsing the output is then byte-stable (a fixpoint).
        let output = card.to_string();
        let reparsed = VcardCst::parse(&output).unwrap();
        assert_eq!(reparsed.to_string(), output);
    }

    #[test]
    fn tolerates_blank_lines_and_a_missing_final_break() {
        // NOTE: a stray blank line after VERSION, and no trailing break after
        // END.
        let input = "BEGIN:VCARD\r\nVERSION:4.0\r\n\r\nFN:John\r\nEND:VCARD";
        let card = VcardCst::parse(input).unwrap();

        assert_eq!(card.decode().properties.len(), 1);
        // NOTE: the blank line is dropped; the missing final break is
        // preserved.
        assert_eq!(
            card.to_string(),
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John\r\nEND:VCARD",
        );
    }
}
