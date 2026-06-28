//! # Content line
//!
//! One raw content line of a card: name, parameters, value, line ending.
//!
//! [`VcardLine`] is the syntactic unit a property occupies. It owns the line
//! tokeniser ([`take`](VcardLine::take), which splits one line off the remaining
//! input for [`VcardCst::parse`](crate::tree::cst::VcardCst::parse)) and the head
//! splitter that separates the name from its parameters. It exposes its raw value
//! and typed parameter access by lens, but stays generic: the meaning of the name
//! and the decoding of the value belong to the lens markers and the
//! [`decode`](crate::tree::decode) / [`encode`](crate::tree::encode) bridges.

use core::fmt;

use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

use crate::{
    error::VcardParseError,
    tree::{leaf::VcardLeaf, lens::VcardParamLens, param::VcardParamNode, value::VcardValueNode},
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
                components: vec![vec![VcardLeaf(value.into())]],
            },
            eol: VcardLeaf(Cow::Borrowed("\r\n")),
        }
    }

    /// Tokenise the line at the start of `rest`, returning it and the remaining
    /// input.
    pub fn take(rest: &'a str) -> Result<(Self, &'a str), VcardParseError> {
        let bytes = rest.as_bytes();

        let Some(lf) = memchr::memchr(b'\n', bytes) else {
            return Err(VcardParseError::MissingCrlf(rest.to_string()));
        };

        let tail = &rest[lf + 1..];

        let (content, eol) = if lf > 0 && bytes[lf - 1] == b'\r' {
            (&rest[..lf - 1], &rest[lf - 1..lf + 1])
        } else {
            (&rest[..lf], &rest[lf..lf + 1])
        };

        let Some(colon) = memchr::memchr(b':', content.as_bytes()) else {
            return Err(VcardParseError::MissingPropertyColon(content.to_string()));
        };

        let line = Self::parse(&content[..colon], &content[colon + 1..], eol);

        Ok((line, tail))
    }

    /// The raw text of the line's first value, for simple single-value lines.
    pub fn raw_value(&self) -> &str {
        self.value
            .components
            .first()
            .and_then(|component| component.first())
            .map(|leaf| leaf.get())
            .unwrap_or("")
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

    /// Parse a head/value/eol triple into a line, splitting the head's params.
    fn parse(head: &'a str, value: &'a str, eol: &'a str) -> Self {
        let (name, params) = split_head(head);

        Self {
            name: VcardLeaf::from(name),
            params,
            value: VcardValueNode::parse(value),
            eol: VcardLeaf::from(eol),
        }
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

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::line::VcardLine;

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
    fn accepts_a_bare_lf_ending() {
        let (line, _) = VcardLine::take("FN:John\n").unwrap();
        assert_eq!(line.to_string(), "FN:John\n");
    }
}
