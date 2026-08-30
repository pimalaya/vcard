//! # Language-tag value codec (RFC 6350 4.8)
//!
//! [`VcardCodec`] for a language-tag value, a single scalar kept as its raw
//! text.

use crate::{
    tree::{
        codec::{VcardCodec, encode::scalar_node, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::language::VcardLanguageTag,
};

impl<'v> VcardCodec<'v> for VcardLanguageTag<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardLanguageTag(node.decode())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
