//! # Parameters (syntax side)
//!
//! The raw parameter node and the per-parameter lens modules.
//!
//! [`VcardParamNode`] is the syntactic peer of the decoded
//! [`VcardParam`](crate::param::VcardParam): a name leaf and its raw value
//! leaves, parsed from and serialized back to the wire verbatim. Each RFC 6350
//! parameter then has its own hand-written lens marker in a submodule here,
//! tying a wire name to the decoded shape (a single value, or a list); those
//! markers are the type-level keys for
//! [`VcardLine::param`](crate::tree::line::VcardLine::param). The name dispatch
//! for whole-line decoding lives in [`crate::tree::codec::decode`].

pub mod altid;
pub mod calscale;
pub mod geo;
pub mod label;
pub mod language;
pub mod mediatype;
pub mod pid;
pub mod pref;
pub mod sort_as;
pub mod r#type;
pub mod tz;
pub mod value;

use core::fmt;

use alloc::vec::Vec;

use crate::{param::VcardParamKind, tree::leaf::VcardLeaf};

/// A parameter identified by type, projecting a generic syntax parameter onto a
/// decoded value type and back.
pub trait VcardParamLens {
    /// The parameter kind to look up by (its wire name comes through `Deref`).
    const KIND: VcardParamKind;

    /// The decoded value type, borrowing the syntax node for reads.
    type Target<'v>;

    /// Project the generic syntax parameter onto the decoded type.
    fn decode<'v>(param: &'v VcardParamNode<'_>) -> Self::Target<'v>;

    /// Encode a decoded value back into a generic syntax parameter (owned).
    fn encode(decoded: &Self::Target<'_>) -> VcardParamNode<'static>;
}

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
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::tree::param::{VcardParamLens, VcardParamNode, language::LANGUAGE, pid::PID};

    #[test]
    fn parses_quoted_values_then_round_trips() {
        let node = VcardParamNode::parse(r#"TYPE=work,"a,b""#);
        assert_eq!(node.values.len(), 2);
        assert_eq!(node.to_string(), r#"TYPE=work,"a,b""#);
    }

    #[test]
    fn decodes_a_list_parameter_through_its_lens() {
        let node = VcardParamNode::parse("PID=1,2");
        assert_eq!(
            PID::decode(&node),
            vec![Cow::Borrowed("1"), Cow::Borrowed("2")],
        );
    }

    #[test]
    fn encodes_a_scalar_parameter_through_its_lens() {
        let node = LANGUAGE::encode(&Cow::Borrowed("en"));
        assert_eq!(node.to_string(), "LANGUAGE=en");
    }
}
