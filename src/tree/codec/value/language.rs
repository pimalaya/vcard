//! # Language-tag value codec (RFC 6350 4.8)
//!
//! [`Codec`] for a language-tag value, a single scalar kept as its raw text.

use crate::{
    tree::{
        codec::{encode::scalar_node, mode::Escaper, value::Codec},
        value::VcardValueNode,
    },
    value::language::VcardLanguageTag,
};

impl<'v> Codec<'v> for VcardLanguageTag<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardLanguageTag(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
