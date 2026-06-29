//! # Value node
//!
//! The raw value of a content line, on the syntax side.
//!
//! [`VcardValueNode`] is the syntactic peer of the decoded
//! [`VcardValue`](crate::v3_0::value::VcardValue): the text after a line's colon,
//! split into `;`-separated components of `,`-separated [`VcardLeaf`] values. The
//! splitting is purely generic (it counts and preserves separators so the value
//! round-trips); what those components *mean* is the lens's business. The codec
//! that turns components into decoded values and back ([`decode_at`], `set_at`,
//! `decode_scalar_at`) lives on this type but is implemented in the
//! [`decode`](crate::v3_0::tree::decode) / [`encode`](crate::v3_0::tree::encode) siblings.
//!
//! [`decode_at`]: VcardValueNode::decode_at

use core::fmt;

use alloc::vec::Vec;

use crate::v3_0::tree::leaf::VcardLeaf;

/// A raw value: `;`-separated components, each a list of `,`-separated raw value
/// leaves. Splitting is generic; joining on serialize restores the bytes.
#[derive(Clone, Debug, Default)]
pub struct VcardValueNode<'a> {
    /// The components, in source order.
    pub components: Vec<Vec<VcardLeaf<'a>>>,
}

impl<'a> VcardValueNode<'a> {
    /// Split a raw value into its components and their values (escape-aware).
    pub fn parse(value: &'a str) -> Self {
        let components = split_components(value)
            .into_iter()
            .map(|component| {
                split_values(component)
                    .into_iter()
                    .map(VcardLeaf::from)
                    .collect()
            })
            .collect();

        Self { components }
    }

    /// Convert into an owned value node (`'static`).
    pub(crate) fn into_static(self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: self
                .components
                .into_iter()
                .map(|component| component.into_iter().map(VcardLeaf::into_static).collect())
                .collect(),
        }
    }
}

impl fmt::Display for VcardValueNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                f.write_str(";")?;
            }
            for (j, leaf) in component.iter().enumerate() {
                if j > 0 {
                    f.write_str(",")?;
                }
                f.write_str(leaf.get())?;
            }
        }

        Ok(())
    }
}

/// Split a value into its `;`-separated components (escape-aware, variable
/// length so counts round-trip).
fn split_components(value: &str) -> Vec<&str> {
    split_unescaped(value, b';')
}

/// Split a component into its `,`-separated values (escape-aware).
fn split_values(component: &str) -> Vec<&str> {
    split_unescaped(component, b',')
}

/// Split on every unescaped `sep`, always yielding at least one piece.
fn split_unescaped(text: &str, sep: u8) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut escaped = false;

    for (i, &byte) in bytes.iter().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == sep {
            pieces.push(&text[start..i]);
            start = i + 1;
        }
    }
    pieces.push(&text[start..]);

    pieces
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::v3_0::tree::value::VcardValueNode;

    #[test]
    fn splits_components_and_values_then_round_trips() {
        let node = VcardValueNode::parse("a;b,c;");
        assert_eq!(node.components.len(), 3);
        assert_eq!(node.components[1].len(), 2);
        assert_eq!(node.to_string(), "a;b,c;");
    }

    #[test]
    fn keeps_escaped_separators_inside_one_value() {
        let node = VcardValueNode::parse(r"a\,b\;c;d");
        assert_eq!(node.components.len(), 2);
        assert_eq!(node.components[0].len(), 1);
        assert_eq!(node.to_string(), r"a\,b\;c;d");
    }
}
