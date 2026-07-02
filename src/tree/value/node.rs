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
    /// Split a raw value into its `;`-separated components, each a list of its
    /// `,`-separated value leaves (escape-aware). Fused and `memchr`-driven: it
    /// jumps to the next separator or backslash instead of scanning every byte,
    /// so a large separator-free value (e.g. base64) is skipped in one pass.
    pub fn parse(value: &'a [u8]) -> Self {
        let mut components = Vec::new();
        split_on(value, b';', |component| {
            components.push(split_component(component));
        });

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

/// Split a component into its `,`-separated value leaves (escape-aware).
fn split_component(component: &[u8]) -> Vec<VcardValueLeaf<'_>> {
    let mut values = Vec::new();
    split_on(component, b',', |value| {
        values.push(VcardValueLeaf::from(value));
    });
    values
}

/// Call `piece` for each span between unescaped `sep` bytes, always at least
/// once. A backslash escapes the next byte (so `\;` / `\,` do not split), and
/// `memchr` skips straight to the next `sep` or backslash instead of scanning
/// byte by byte.
fn split_on<'b>(bytes: &'b [u8], sep: u8, mut piece: impl FnMut(&'b [u8])) {
    let mut start = 0;
    let mut i = 0;

    while let Some(offset) = memchr::memchr2(b'\\', sep, &bytes[i..]) {
        let pos = i + offset;
        if bytes[pos] == b'\\' {
            i = (pos + 2).min(bytes.len());
        } else {
            piece(&bytes[start..pos]);
            start = pos + 1;
            i = pos + 1;
        }
    }

    piece(&bytes[start..]);
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
