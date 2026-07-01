//! # Content line
//!
//! One raw content line of a card: name, parameters, value, line ending.
//!
//! [`VcardLine`] is the syntactic unit a property occupies. It owns the line
//! tokeniser ([`take`](VcardLine::take), which splits one logical line off the
//! remaining input for [`VcardCst::parse`](crate::tree::cst::VcardCst::parse),
//! unfolding any RFC 6350 3.2 folded continuation lines) and the head splitter
//! that separates the name from its parameters. It exposes its raw value and
//! typed parameter access by lens, but stays generic: the meaning of the name
//! and the decoding of the value belong to the lens markers and the
//! [`decode`](crate::tree::codec::decode) /
//! [`encode`](crate::tree::codec::encode) bridges.
//!
//! Folding and stray blank lines are normalised away on parse, not preserved: a
//! folded line unfolds to its logical content, blank lines are dropped, and the
//! final line needs no trailing break. A clean, unfolded card still round-trips
//! byte for byte.

use core::{fmt, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::tree::{
    codec::mode::Escaper,
    error::VcardParseError,
    leaf::{VcardLeaf, VcardValueLeaf},
    param::{VcardParamLens, VcardParamNode},
    value::VcardValueNode,
};

/// One raw content line: a name, parameters, a value and the line ending.
///
/// This is a *logical* line, not a physical one: [`take`](Self::take) unfolds
/// RFC 6350 3.2 folded continuations and QUOTED-PRINTABLE soft line breaks, so
/// a `VcardLine` never holds an internal line break, only its terminating
/// [`eol`](Self::eol). It is also the syntactic unit for the `BEGIN` /
/// `VERSION` / `END` envelope lines, not only decoded properties.
#[derive(Clone, Debug)]
pub struct VcardLine<'a> {
    /// The property name leaf, with any group prefix.
    pub name: VcardLeaf<'a>,
    /// The parameters, in source order.
    pub params: Vec<VcardParamNode<'a>>,
    /// The value.
    pub value: VcardValueNode<'a>,
    /// The line ending (`\r\n` or `\n`).
    pub eol: VcardLeaf<'a>,
}

impl<'a> VcardLine<'a> {
    /// Build a property line with a raw text value and the default `\r\n`
    /// ending. Used to seed BEGIN/VERSION/END and to encode simple values.
    pub fn text(name: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: VcardLeaf(name.into()),
            params: Vec::new(),
            value: VcardValueNode {
                escaper: Escaper::Modern,
                components: vec![vec![VcardValueLeaf::from(value.into())]],
            },
            eol: VcardLeaf(Cow::Borrowed("\r\n")),
        }
    }

    /// Tokenise the logical line at the start of `rest`, unfolding any folded
    /// continuation lines, and return it with the remaining input. RFC 6350 3.2
    /// folds a long line by inserting a CRLF and a single leading space or tab;
    /// unfolding drops them. A line with no folds borrows the source; a folded
    /// line is rebuilt owned, since its bytes are no longer contiguous.
    pub fn take(rest: &'a [u8]) -> Result<(Self, &'a [u8]), VcardParseError> {
        // NOTE: skip blank lines: real-world exports sometimes emit them.
        let mut head = rest;
        let (first, eol, mut tail) = loop {
            if head.is_empty() {
                return Err(VcardParseError::MissingCrlf(lossy(rest)));
            }
            let (content, eol, next) = physical_line(head);
            if content.is_empty() {
                head = next;
                continue;
            }
            break (content, eol, next);
        };

        // NOTE: QUOTED-PRINTABLE soft line breaks: a line whose head declares
        // ENCODING=QUOTED-PRINTABLE and whose value ends with `=` continues on
        // the next physical line. Param-driven, so it applies to any version's
        // card that uses the encoding, not just 2.1.
        if first.ends_with(b"=") && head_is_quoted_printable(first) {
            let mut logical = Vec::from(&first[..first.len() - 1]);
            let mut last_eol;
            loop {
                let (continuation, eol, next) = physical_line(tail);
                last_eol = eol;
                tail = next;
                match continuation.strip_suffix(b"=") {
                    Some(head) => logical.extend_from_slice(head),
                    None => {
                        logical.extend_from_slice(continuation);
                        break;
                    }
                }
                if tail.is_empty() {
                    break;
                }
            }
            let mut line = Self::parse(&logical, b"")?.into_static();
            line.eol = eol_leaf(last_eol);
            return Ok((line, tail));
        }

        if !starts_with_wsp(tail) {
            return Ok((Self::parse(first, eol)?, tail));
        }

        let mut logical = Vec::from(first);
        let mut last_eol = eol;

        while starts_with_wsp(tail) {
            let (continuation, eol, next) = physical_line(&tail[1..]);
            logical.extend_from_slice(continuation);
            last_eol = eol;
            tail = next;
        }

        let mut line = Self::parse(&logical, b"")?.into_static();
        line.eol = eol_leaf(last_eol);

        Ok((line, tail))
    }

    /// Convert into an owned line whose every leaf is owned (`'static`).
    pub(crate) fn into_static(self) -> VcardLine<'static> {
        VcardLine {
            name: self.name.into_static(),
            params: self
                .params
                .into_iter()
                .map(VcardParamNode::into_static)
                .collect(),
            value: self.value.into_static(),
            eol: self.eol.into_static(),
        }
    }

    /// The raw bytes of the line's first value, for simple single-value lines.
    pub fn raw_value(&self) -> &[u8] {
        self.value
            .components
            .first()
            .and_then(|component| component.first())
            .map(|leaf| leaf.as_bytes())
            .unwrap_or(b"")
    }

    /// The raw first value as UTF-8 text, lossily; for the ASCII envelope
    /// values (`VERSION`) and diagnostics.
    pub fn raw_value_str(&self) -> Cow<'_, str> {
        self.value
            .components
            .first()
            .and_then(|component| component.first())
            .map(|leaf| leaf.to_str_lossy())
            .unwrap_or(Cow::Borrowed(""))
    }

    /// Serialize the whole line to bytes, exactly as parsed.
    pub(crate) fn write_bytes(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.name.get().as_bytes());

        for param in &self.params {
            out.push(b';');
            out.extend_from_slice(param.to_string().as_bytes());
        }

        out.push(b':');
        self.value.write_bytes(out);
        out.extend_from_slice(self.eol.get().as_bytes());
    }

    /// The first parameter of type `P`, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.params
            .iter()
            .find(|param| param.name.get().eq_ignore_ascii_case(&P::KIND))
            .map(|param| P::decode(param))
    }

    /// The first parameter of type `P`, mutably (raw, for editing its leaves).
    pub fn param_mut<P: VcardParamLens>(&mut self) -> Option<&mut VcardParamNode<'a>> {
        self.params
            .iter_mut()
            .find(|param| param.name.get().eq_ignore_ascii_case(&P::KIND))
    }

    /// Split one logical line into a typed line at the colon, separating the
    /// name, its parameters and the value. The head (name and parameters) must
    /// be valid UTF-8, as every version's grammar guarantees; only the value
    /// may carry a foreign charset, so it is kept as raw bytes.
    fn parse<'b>(content: &'b [u8], eol: &'b [u8]) -> Result<VcardLine<'b>, VcardParseError> {
        let Some(colon) = content.iter().position(|&b| b == b':') else {
            return Err(VcardParseError::MissingPropertyColon(lossy(content)));
        };

        let head = str::from_utf8(&content[..colon])
            .map_err(|_| VcardParseError::NonUtf8Header(lossy(&content[..colon])))?;
        let (name, params) = split_head(head);

        Ok(VcardLine {
            name: VcardLeaf::from(name),
            params,
            value: VcardValueNode::parse(&content[colon + 1..]),
            eol: VcardLeaf::from(str::from_utf8(eol).unwrap_or("")),
        })
    }
}

impl fmt::Display for VcardLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.get())?;

        for param in &self.params {
            write!(f, ";{param}")?;
        }

        write!(f, ":{}{}", self.value, self.eol.get())
    }
}

/// Split a head into its name and its `;`-separated parameters.
fn split_head(head: &str) -> (&str, Vec<VcardParamNode<'_>>) {
    let (name, mut rest) = match head.find(';') {
        Some(semi) => (&head[..semi], &head[semi..]),
        None => return (head, Vec::new()),
    };

    let mut params = Vec::new();

    while let Some(after) = rest.strip_prefix(';') {
        let (param, tail) = match after.find(';') {
            Some(semi) => (&after[..semi], &after[semi..]),
            None => (after, ""),
        };

        params.push(VcardParamNode::parse(param));
        rest = tail;
    }

    (name, params)
}

/// Split the first physical line off `rest`: its content (without the line
/// ending), its line ending, and the remaining input. A final line with no
/// trailing break is taken whole, with an empty ending.
fn physical_line(rest: &[u8]) -> (&[u8], &[u8], &[u8]) {
    let Some(lf) = rest.iter().position(|&b| b == b'\n') else {
        return (rest, b"", b"");
    };

    let tail = &rest[lf + 1..];

    let (content, eol) = if lf > 0 && rest[lf - 1] == b'\r' {
        (&rest[..lf - 1], &rest[lf - 1..lf + 1])
    } else {
        (&rest[..lf], &rest[lf..lf + 1])
    };

    (content, eol, tail)
}

/// Whether `rest` begins with a folding whitespace (space or tab).
fn starts_with_wsp(rest: &[u8]) -> bool {
    matches!(rest.first(), Some(b' ' | b'\t'))
}

/// Whether a line's head (its name and parameters, before the `:`) declares the
/// `QUOTED-PRINTABLE` encoding, as an `ENCODING=` parameter or a bare token.
fn head_is_quoted_printable(line: &[u8]) -> bool {
    let head = match line.iter().position(|&b| b == b':') {
        Some(colon) => &line[..colon],
        None => return false,
    };

    head.split(|&b| b == b';').any(|token| {
        token.eq_ignore_ascii_case(b"QUOTED-PRINTABLE")
            || token.eq_ignore_ascii_case(b"ENCODING=QUOTED-PRINTABLE")
    })
}

/// An owned line-ending leaf from raw bytes (always an ASCII `\r\n` / `\n`).
fn eol_leaf(bytes: &[u8]) -> VcardLeaf<'static> {
    VcardLeaf::from(String::from_utf8_lossy(bytes).into_owned())
}

/// A lossy owned string of raw bytes, for error diagnostics.
fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::line::VcardLine;

    #[test]
    fn takes_one_line_and_leaves_the_rest() {
        let (line, rest) = VcardLine::take(b"FN:John\r\nEND:VCARD\r\n").unwrap();
        assert_eq!(line.name.get(), "FN");
        assert_eq!(line.to_string(), "FN:John\r\n");
        assert_eq!(rest, b"END:VCARD\r\n");
    }

    #[test]
    fn splits_parameters_off_the_head_then_round_trips() {
        let (line, _) = VcardLine::take(b"TEL;TYPE=work,home:123\r\n").unwrap();
        assert_eq!(line.params.len(), 1);
        assert_eq!(line.to_string(), "TEL;TYPE=work,home:123\r\n");
    }

    #[test]
    fn accepts_a_bare_lf_ending() {
        let (line, _) = VcardLine::take(b"FN:John\n").unwrap();
        assert_eq!(line.to_string(), "FN:John\n");
    }

    #[test]
    fn unfolds_space_and_tab_continuations() {
        let (line, rest) = VcardLine::take(b"NOTE:foo\r\n bar\r\n\tbaz\r\nEND:VCARD\r\n").unwrap();
        assert_eq!(line.name.get(), "NOTE");
        assert_eq!(line.raw_value_str(), "foobarbaz");
        assert_eq!(rest, b"END:VCARD\r\n");
    }

    #[test]
    fn serializes_an_unfolded_line() {
        let (line, _) = VcardLine::take(b"NOTE:foo\r\n bar\r\n").unwrap();
        assert_eq!(line.to_string(), "NOTE:foobar\r\n");
    }

    #[test]
    fn keeps_whitespace_beyond_the_single_fold_indicator() {
        // NOTE: only the first space is the fold marker; the rest is value
        // content.
        let (line, _) = VcardLine::take(b"NOTE:foo\r\n  bar\r\n").unwrap();
        assert_eq!(line.raw_value_str(), "foo bar");
    }

    #[test]
    fn skips_blank_lines_before_the_next_line() {
        let (line, rest) = VcardLine::take(b"\r\n\r\nFN:John\r\nEND:VCARD\r\n").unwrap();
        assert_eq!(line.name.get(), "FN");
        assert_eq!(rest, b"END:VCARD\r\n");
    }

    #[test]
    fn tolerates_a_missing_final_line_break() {
        let (line, rest) = VcardLine::take(b"END:VCARD").unwrap();
        assert_eq!(line.name.get(), "END");
        assert_eq!(line.to_string(), "END:VCARD");
        assert_eq!(rest, b"");
    }
}
