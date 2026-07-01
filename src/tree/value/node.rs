//! # Value node
//!
//! The raw value of a content line, on the syntax side.
//!
//! [`VcardValueNode`] is the syntactic peer of the decoded
//! [`VcardValue`](crate::value::VcardValue): the bytes after a line's colon,
//! split into `;`-separated components of `,`-separated
//! [`VcardValueLeaf`](crate::tree::leaf::VcardValueLeaf) values (raw bytes, so
//! a foreign charset survives). The splitting is purely generic (it counts and
//! preserves separators so the value round-trips); what those components *mean*
//! is the lens's business. The codec that turns components into decoded values
//! and back ([`decode_at`], `set_at`, `decode_scalar_at`) lives on this type
//! but is implemented in the [`decode`](crate::tree::codec::decode) /
//! [`encode`](crate::tree::codec::encode) siblings.
//!
//! [`decode_at`]: VcardValueNode::decode_at

use core::fmt;

use alloc::{string::String, vec::Vec};

use crate::tree::{codec::mode::Escaper, leaf::VcardValueLeaf};

/// A raw value: `;`-separated components, each a list of `,`-separated raw
/// value leaves. Splitting is generic; joining on serialize restores the
/// bytes. The `escaper` records which version's escaping rules the codec must
/// apply; it is stamped from the card version after parsing (see
/// `VcardCst::parse`).
#[derive(Clone, Debug, Default)]
pub struct VcardValueNode<'a> {
    /// The components, in source order.
    pub components: Vec<Vec<VcardValueLeaf<'a>>>,
    /// The escaping rules to read and write this value with.
    pub escaper: Escaper,
}

impl<'a> VcardValueNode<'a> {
    /// Split a raw value into its components and their values (escape-aware).
    pub fn parse(value: &'a [u8]) -> Self {
        let components = split_components(value)
            .into_iter()
            .map(|component| {
                split_values(component)
                    .into_iter()
                    .map(VcardValueLeaf::from)
                    .collect()
            })
            .collect();

        Self {
            components,
            escaper: Escaper::default(),
        }
    }

    /// Serialize the raw value bytes (name, colon and eol are the line's job)
    /// into `out`, exactly as parsed.
    pub(crate) fn write_bytes(&self, out: &mut Vec<u8>) {
        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                out.push(b';');
            }

            for (j, leaf) in component.iter().enumerate() {
                if j > 0 {
                    out.push(b',');
                }

                out.extend_from_slice(leaf.as_bytes());
            }
        }
    }

    /// Convert into an owned value node (`'static`).
    pub(crate) fn into_static(self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: self
                .components
                .into_iter()
                .map(|component| {
                    component
                        .into_iter()
                        .map(VcardValueLeaf::into_static)
                        .collect()
                })
                .collect(),
            escaper: self.escaper,
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

                f.write_str(&String::from_utf8_lossy(leaf.as_bytes()))?;
            }
        }

        Ok(())
    }
}

/// Split a value into its `;`-separated components (escape-aware, variable
/// length so counts round-trip).
fn split_components(value: &[u8]) -> Vec<&[u8]> {
    split_unescaped(value, b';')
}

/// Split a component into its `,`-separated values (escape-aware).
fn split_values(component: &[u8]) -> Vec<&[u8]> {
    split_unescaped(component, b',')
}

/// Split on every unescaped `sep`, always yielding at least one piece.
fn split_unescaped(bytes: &[u8], sep: u8) -> Vec<&[u8]> {
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut escaped = false;

    for (i, &byte) in bytes.iter().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == sep {
            pieces.push(&bytes[start..i]);
            start = i + 1;
        }
    }
    pieces.push(&bytes[start..]);

    pieces
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::value::VcardValueNode;

    #[test]
    fn splits_components_and_values_then_round_trips() {
        let node = VcardValueNode::parse(b"a;b,c;");
        assert_eq!(node.components.len(), 3);
        assert_eq!(node.components[1].len(), 2);
        assert_eq!(node.to_string(), "a;b,c;");
    }

    #[test]
    fn keeps_escaped_separators_inside_one_value() {
        let node = VcardValueNode::parse(br"a\,b\;c;d");
        assert_eq!(node.components.len(), 2);
        assert_eq!(node.components[0].len(), 1);
        assert_eq!(node.to_string(), r"a\,b\;c;d");
    }
}
