//! # Content line
//!
//! One raw content line of a card: name, parameters, value, line ending.
//!
//! [`VcardLine`] is the syntactic unit a property occupies. It owns the line
//! tokeniser ([`take`](VcardLine::take), which splits one logical line off the
//! remaining input for [`VcardCst::parse`](crate::v2_1::tree::cst::VcardCst::parse),
//! unfolding any folded continuation lines) and the head splitter that
//! separates the name from its parameters. It exposes its raw value and typed
//! parameter access by lens, but stays generic: the meaning of the name and the
//! decoding of the value belong to the lens markers and the
//! [`decode`](crate::v2_1::tree::decode) / [`encode`](crate::v2_1::tree::encode) bridges.
//!
//! Folding and stray blank lines are normalised away on parse, not preserved: a
//! folded line unfolds to its logical content, blank lines are dropped, and the
//! final line needs no trailing break. A clean, unfolded card still round-trips
//! byte for byte.

use core::fmt;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::v2_1::tree::{
    error::VcardParseError, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode,
    value::VcardValueNode,
};

/// One raw content line: a name, parameters, a value and the line ending.
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
                components: vec![VcardLeaf(value.into())],
            },
            eol: VcardLeaf(Cow::Borrowed("\r\n")),
        }
    }

    /// Tokenise the logical line at the start of `rest`, unfolding any folded
    /// continuation lines, and return it with the remaining input. vCard 2.1
    /// folds a long line by inserting a CRLF and a single leading space or tab;
    /// unfolding drops them. A line with no folds borrows the source; a folded
    /// line is rebuilt owned, since its bytes are no longer contiguous.
    pub fn take(rest: &'a str) -> Result<(Self, &'a str), VcardParseError> {
        // Skip blank lines: real-world exports sometimes emit them.
        let mut head = rest;
        let (first, eol, mut tail) = loop {
            if head.is_empty() {
                return Err(VcardParseError::MissingCrlf(rest.to_string()));
            }
            let (content, eol, next) = physical_line(head);
            if content.is_empty() {
                head = next;
                continue;
            }
            break (content, eol, next);
        };

        if first.ends_with('=') && head_is_quoted_printable(first) {
            let mut logical = String::from(&first[..first.len() - 1]);
            let mut last_eol;
            loop {
                let (continuation, eol, next) = physical_line(tail);
                last_eol = eol;
                tail = next;
                match continuation.strip_suffix('=') {
                    Some(head) => logical.push_str(head),
                    None => {
                        logical.push_str(continuation);
                        break;
                    }
                }
                if tail.is_empty() {
                    break;
                }
            }
            let mut line = Self::parse(&logical, "")?.into_static();
            line.eol = VcardLeaf::from(last_eol.to_string());
            return Ok((line, tail));
        }

        if !starts_with_wsp(tail) {
            return Ok((Self::parse(first, eol)?, tail));
        }

        let mut logical = String::from(first);
        let mut last_eol = eol;

        while starts_with_wsp(tail) {
            let (continuation, eol, next) = physical_line(&tail[1..]);
            logical.push_str(continuation);
            last_eol = eol;
            tail = next;
        }

        let mut line = Self::parse(&logical, "")?.into_static();
        line.eol = VcardLeaf::from(last_eol.to_string());

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

    /// The raw text of the line's first value, for simple single-value lines.
    pub fn raw_value(&self) -> &str {
        self.value
            .components
            .first()
            .map(|leaf| leaf.get())
            .unwrap_or("")
    }

    /// The whole raw value: all `;`-components rejoined verbatim (no unescaping).
    /// For non-compound values, whose `;` is a literal character rather than a
    /// component separator (text, URIs, base64).
    pub fn raw_value_full(&self) -> Cow<'_, str> {
        match self.value.components.as_slice() {
            [] => Cow::Borrowed(""),
            [one] => Cow::Borrowed(one.get()),
            many => {
                let mut joined = String::new();

                for (i, leaf) in many.iter().enumerate() {
                    if i > 0 {
                        joined.push(';');
                    }
                    joined.push_str(leaf.get());
                }

                Cow::Owned(joined)
            }
        }
    }

    /// The first parameter of type `P`, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.params
            .iter()
            .find(|param| param.name.get().eq_ignore_ascii_case(P::NAME))
            .map(|param| P::decode(param))
    }

    /// The first parameter of type `P`, mutably (raw, for editing its leaves).
    pub fn param_mut<P: VcardParamLens>(&mut self) -> Option<&mut VcardParamNode<'a>> {
        self.params
            .iter_mut()
            .find(|param| param.name.get().eq_ignore_ascii_case(P::NAME))
    }

    /// Split one logical line into a typed line at the colon, separating the
    /// name, its parameters and the value.
    fn parse<'b>(content: &'b str, eol: &'b str) -> Result<VcardLine<'b>, VcardParseError> {
        let Some(colon) = memchr::memchr(b':', content.as_bytes()) else {
            return Err(VcardParseError::MissingPropertyColon(content.to_string()));
        };

        let (name, params) = split_head(&content[..colon]);

        Ok(VcardLine {
            name: VcardLeaf::from(name),
            params,
            value: VcardValueNode::parse(&content[colon + 1..]),
            eol: VcardLeaf::from(eol),
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

/// Split a head into its name and its `;`-separated parameters, treating `\;` as
/// an escaped literal rather than a separator.
fn split_head(head: &str) -> (&str, Vec<VcardParamNode<'_>>) {
    let (name, mut rest) = match find_unescaped(head, b';') {
        Some(semi) => (&head[..semi], &head[semi..]),
        None => return (head, Vec::new()),
    };

    let mut params = Vec::new();

    while let Some(after) = rest.strip_prefix(';') {
        let (param, tail) = match find_unescaped(after, b';') {
            Some(semi) => (&after[..semi], &after[semi..]),
            None => (after, ""),
        };

        params.push(VcardParamNode::parse(param));
        rest = tail;
    }

    (name, params)
}

/// The index of the first unescaped `target` byte, where a `\` escapes the byte
/// that follows it.
fn find_unescaped(text: &str, target: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut escaped = false;

    for (i, &byte) in bytes.iter().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == target {
            return Some(i);
        }
    }

    None
}

/// Split the first physical line off `rest`: its content (without the line
/// ending), its line ending, and the remaining input. A final line with no
/// trailing break is taken whole, with an empty ending.
fn physical_line(rest: &str) -> (&str, &str, &str) {
    let bytes = rest.as_bytes();

    let Some(lf) = memchr::memchr(b'\n', bytes) else {
        return (rest, "", "");
    };

    let tail = &rest[lf + 1..];

    let (content, eol) = if lf > 0 && bytes[lf - 1] == b'\r' {
        (&rest[..lf - 1], &rest[lf - 1..lf + 1])
    } else {
        (&rest[..lf], &rest[lf..lf + 1])
    };

    (content, eol, tail)
}

/// Whether `rest` begins with a folding whitespace (space or tab).
fn starts_with_wsp(rest: &str) -> bool {
    matches!(rest.as_bytes().first(), Some(b' ' | b'\t'))
}

/// Whether a physical line's head (before the colon) declares
/// `QUOTED-PRINTABLE`, written either as an `ENCODING=` parameter or as a bare
/// 2.1 token. The raw-string peer of `VcardLine::is_quoted_printable` (in the
/// `decode` module), run before the line is split into parameters.
fn head_is_quoted_printable(line: &str) -> bool {
    let head = match memchr::memchr(b':', line.as_bytes()) {
        Some(colon) => &line[..colon],
        None => return false,
    };

    head.split(';').any(|token| {
        token.eq_ignore_ascii_case("QUOTED-PRINTABLE")
            || token.eq_ignore_ascii_case("ENCODING=QUOTED-PRINTABLE")
    })
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::v2_1::tree::line::VcardLine;

    #[test]
    fn takes_one_line_and_leaves_the_rest() {
        let (line, rest) = VcardLine::take("FN:John\r\nEND:VCARD\r\n").unwrap();
        assert_eq!(line.name.get(), "FN");
        assert_eq!(line.to_string(), "FN:John\r\n");
        assert_eq!(rest, "END:VCARD\r\n");
    }

    #[test]
    fn splits_parameters_off_the_head_then_round_trips() {
        let (line, _) = VcardLine::take("TEL;TYPE=work,home:123\r\n").unwrap();
        assert_eq!(line.params.len(), 1);
        assert_eq!(line.to_string(), "TEL;TYPE=work,home:123\r\n");
    }

    #[test]
    fn keeps_an_escaped_semicolon_inside_a_parameter_value() {
        let (line, _) = VcardLine::take("TEL;TYPE=a\\;b:123\r\n").unwrap();
        assert_eq!(line.params.len(), 1);
        assert_eq!(line.to_string(), "TEL;TYPE=a\\;b:123\r\n");
    }

    #[test]
    fn accepts_a_bare_lf_ending() {
        let (line, _) = VcardLine::take("FN:John\n").unwrap();
        assert_eq!(line.to_string(), "FN:John\n");
    }

    #[test]
    fn unfolds_space_and_tab_continuations() {
        let (line, rest) = VcardLine::take("NOTE:foo\r\n bar\r\n\tbaz\r\nEND:VCARD\r\n").unwrap();
        assert_eq!(line.name.get(), "NOTE");
        assert_eq!(line.raw_value(), "foobarbaz");
        assert_eq!(rest, "END:VCARD\r\n");
    }

    #[test]
    fn serializes_an_unfolded_line() {
        let (line, _) = VcardLine::take("NOTE:foo\r\n bar\r\n").unwrap();
        assert_eq!(line.to_string(), "NOTE:foobar\r\n");
    }

    #[test]
    fn keeps_whitespace_beyond_the_single_fold_indicator() {
        // only the first space is the fold marker; the rest is value content.
        let (line, _) = VcardLine::take("NOTE:foo\r\n  bar\r\n").unwrap();
        assert_eq!(line.raw_value(), "foo bar");
    }

    #[test]
    fn joins_quoted_printable_soft_line_breaks() {
        let (line, rest) =
            VcardLine::take("NOTE;ENCODING=QUOTED-PRINTABLE:Hello=\r\nWorld\r\nEND:VCARD\r\n")
                .unwrap();
        assert_eq!(line.raw_value(), "HelloWorld");
        assert_eq!(rest, "END:VCARD\r\n");
    }

    #[test]
    fn skips_blank_lines_before_the_next_line() {
        let (line, rest) = VcardLine::take("\r\n\r\nFN:John\r\nEND:VCARD\r\n").unwrap();
        assert_eq!(line.name.get(), "FN");
        assert_eq!(rest, "END:VCARD\r\n");
    }

    #[test]
    fn tolerates_a_missing_final_line_break() {
        let (line, rest) = VcardLine::take("END:VCARD").unwrap();
        assert_eq!(line.name.get(), "END");
        assert_eq!(line.to_string(), "END:VCARD");
        assert_eq!(rest, "");
    }
}
