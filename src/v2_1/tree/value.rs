//! # Value node
//!
//! The raw value of a content line, on the syntax side.
//!
//! [`VcardValueNode`] is the syntactic peer of the decoded
//! [`VcardValue`](crate::v2_1::value::VcardValue): the text after a line's colon.
//! In vCard 2.1 a value is a `;`-separated list of **single** values (2.1 has no
//! comma sub-lists within a component), so the node is a flat list of
//! [`VcardLeaf`] components. The splitting is escape-aware (`\;` keeps a literal
//! semicolon inside a component) and generic; what the components *mean* is the
//! lens's business, and the codec that cooks them (unescaping, quoted-printable)
//! lives in [`decode`](crate::v2_1::tree::decode) / [`encode`](crate::v2_1::tree::encode).

use core::fmt;

use alloc::vec::Vec;

use crate::v2_1::tree::leaf::VcardLeaf;

/// A raw value: `;`-separated single-value components. Splitting is generic;
/// joining on serialize restores the bytes.
#[derive(Clone, Debug, Default)]
pub struct VcardValueNode<'a> {
    /// The `;`-separated components, in source order.
    pub components: Vec<VcardLeaf<'a>>,
}

impl<'a> VcardValueNode<'a> {
    /// Split a raw value into its `;`-separated components (escape-aware).
    pub fn parse(value: &'a str) -> Self {
        let components = split_components(value)
            .into_iter()
            .map(VcardLeaf::from)
            .collect();
        Self { components }
    }

    /// Convert into an owned value node (`'static`).
    pub(crate) fn into_static(self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: self
                .components
                .into_iter()
                .map(VcardLeaf::into_static)
                .collect(),
        }
    }
}

impl fmt::Display for VcardValueNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, leaf) in self.components.iter().enumerate() {
            if i > 0 {
                f.write_str(";")?;
            }
            f.write_str(leaf.get())?;
        }

        Ok(())
    }
}

/// Split a value on every unescaped `;`, always yielding at least one piece. A
/// `\;` keeps a literal semicolon inside a component.
fn split_components(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut escaped = false;

    for (i, &byte) in bytes.iter().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b';' {
            pieces.push(&value[start..i]);
            start = i + 1;
        }
    }
    pieces.push(&value[start..]);

    pieces
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::v2_1::tree::value::VcardValueNode;

    #[test]
    fn splits_components_then_round_trips() {
        let node = VcardValueNode::parse("Doe;John;;;");
        assert_eq!(node.components.len(), 5);
        assert_eq!(node.to_string(), "Doe;John;;;");
    }

    #[test]
    fn keeps_an_escaped_semicolon_inside_a_component() {
        let node = VcardValueNode::parse(r"a\;b;c");
        assert_eq!(node.components.len(), 2);
        assert_eq!(node.to_string(), r"a\;b;c");
    }
}
