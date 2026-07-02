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

use crate::tree::leaf::VcardLeaf;

/// One raw parameter: a name and its `,`-separated raw values (empty when the
/// parameter has no `=` list). The syntactic peer of the decoded
/// [`VcardParam`](crate::param::VcardParam).
#[derive(Clone, Debug)]
pub struct VcardParamNode<'a> {
    /// The parameter name leaf.
    pub name: VcardLeaf<'a>,
    /// The raw value leaves.
    pub values: Vec<VcardLeaf<'a>>,
}

impl<'a> VcardParamNode<'a> {
    /// Parse one `name=value,value` parameter (commas outside quotes split).
    pub fn parse(param: &'a str) -> Self {
        match param.split_once('=') {
            Some((name, values)) => Self {
                name: VcardLeaf::from(name),
                values: split_param_values(values)
                    .into_iter()
                    .map(VcardLeaf::from)
                    .collect(),
            },
            None => Self {
                name: VcardLeaf::from(param),
                values: Vec::new(),
            },
        }
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
fn split_param_values(values: &str) -> Vec<&str> {
    let bytes = values.as_bytes();
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut quoted = false;

    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                pieces.push(&values[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    pieces.push(&values[start..]);
    pieces
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::param::VcardParamNode;

    #[test]
    fn parses_quoted_values_then_round_trips() {
        let node = VcardParamNode::parse(r#"TYPE=work,"a,b""#);
        assert_eq!(node.values.len(), 2);
        assert_eq!(node.to_string(), r#"TYPE=work,"a,b""#);
    }
}
