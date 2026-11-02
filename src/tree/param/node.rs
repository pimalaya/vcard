//! # Parameter node
//!
//! The raw, byte-faithful parameter on the syntax side.
//!
//! [`VcardParamNode`] is the syntactic peer of the decoded
//! [`VcardParam`](crate::param::VcardParam): a name leaf and its raw value
//! leaves, parsed from and serialized back to the wire verbatim. The per-name
//! lens markers that give it meaning live alongside in [`crate::tree::param`].

use core::fmt;

use alloc::vec::Vec;

use crate::tree::{codec::mode::VcardEscaper, leaf::VcardLeaf};

/// One raw parameter: a name and its `,`-separated raw values (empty when the
/// parameter has no `=` list). The syntactic peer of the decoded
/// [`VcardParam`](crate::param::VcardParam).
#[derive(Clone, Debug)]
pub struct VcardParamNode<'a> {
    /// The parameter name leaf.
    pub name: VcardLeaf<'a>,
    /// The raw value leaves.
    pub values: Vec<VcardLeaf<'a>>,
    /// Which version's parameter encoding rules the values are written in.
    /// Stamped by the parser once `VERSION` is known, as on a value node.
    pub escaper: VcardEscaper,
}

impl<'a> VcardParamNode<'a> {
    /// Parse one `name=value,value` parameter (commas outside quotes split).
    pub fn parse(param: &'a str) -> Self {
        match param.split_once('=') {
            Some((name, values)) => Self {
                name: VcardLeaf::from(name),
                values: split_param_values(values),
                escaper: VcardEscaper::default(),
            },
            None => Self {
                name: VcardLeaf::from(param),
                values: Vec::new(),
                escaper: VcardEscaper::default(),
            },
        }
    }

    /// Take one parameter off the front of `rest`, the head bytes after a
    /// `;`, up to the next `;` outside double quotes: the node, then what
    /// follows that `;`, if one ends it.
    ///
    /// One walk does what [`parse`](Self::parse) does after the head splitter
    /// found the `;`: the first `=` ends the name, and a `,` outside quotes
    /// counted from that `=` on splits two values.
    pub(crate) fn take(rest: &'a str) -> (Self, Option<&'a str>) {
        let mut quoted = false;
        let mut values_quoted = false;
        let mut eq = None;
        let mut start = 0;
        let mut values = Vec::new();
        let mut end = rest.len();
        let mut tail = None;

        for (i, &byte) in rest.as_bytes().iter().enumerate() {
            match byte {
                b'"' => {
                    quoted = !quoted;
                    values_quoted = !values_quoted;
                }
                b';' if !quoted => {
                    end = i;
                    tail = Some(&rest[i + 1..]);
                    break;
                }
                b'=' if eq.is_none() => {
                    eq = Some(i);
                    start = i + 1;
                    values_quoted = false;
                }
                b',' if eq.is_some() && !values_quoted => {
                    values.push(VcardLeaf::from(&rest[start..i]));
                    start = i + 1;
                }
                _ => {}
            }
        }

        let name = match eq {
            Some(eq) => {
                values.push(VcardLeaf::from(&rest[start..end]));
                &rest[..eq]
            }
            None => &rest[..end],
        };

        let node = Self {
            name: VcardLeaf::from(name),
            values,
            escaper: VcardEscaper::default(),
        };

        (node, tail)
    }

    /// Convert into an owned parameter node (`'static`).
    pub(crate) fn into_static(self) -> VcardParamNode<'static> {
        VcardParamNode {
            name: self.name.into_static(),
            values: self
                .values
                .into_iter()
                .map(VcardLeaf::into_static)
                .collect(),
            escaper: self.escaper,
        }
    }

    /// Serialize the parameter (`name` or `name=value,value`) into `out`,
    /// without the intermediate `String` a `Display`-based path would allocate.
    pub(crate) fn write_bytes(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.name.get().as_bytes());

        if let Some((first, rest)) = self.values.split_first() {
            out.push(b'=');
            out.extend_from_slice(first.get().as_bytes());
            for value in rest {
                out.push(b',');
                out.extend_from_slice(value.get().as_bytes());
            }
        }
    }
}

impl fmt::Display for VcardParamNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.get())?;

        if let Some((first, rest)) = self.values.split_first() {
            write!(f, "={}", first.get())?;

            for value in rest {
                write!(f, ",{}", value.get())?;
            }
        }

        Ok(())
    }
}

/// Split a parameter value list on commas outside double quotes.
fn split_param_values(values: &str) -> Vec<VcardLeaf<'_>> {
    let bytes = values.as_bytes();
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut quoted = false;

    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                pieces.push(VcardLeaf::from(&values[start..i]));
                start = i + 1;
            }
            _ => {}
        }
    }

    pieces.push(VcardLeaf::from(&values[start..]));
    pieces
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::param::node::VcardParamNode;

    #[test]
    fn parses_quoted_values_then_round_trips() {
        let node = VcardParamNode::parse(r#"TYPE=work,"a,b""#);
        assert_eq!(node.values.len(), 2);
        assert_eq!(node.to_string(), r#"TYPE=work,"a,b""#);
    }
}
