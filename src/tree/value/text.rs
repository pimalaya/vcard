//! # Text value codec (RFC 6350 4.1)
//!
//! [`Codec`] for a single text value and a comma-separated text list.

use alloc::vec;

use crate::{
    tree::{
        codec::{
            Codec,
            encode::{encode_component, scalar_node},
            mode::Escaper,
        },
        value::VcardValueNode,
    },
    value::text::{VcardText, VcardTextList},
};

impl<'v> Codec<'v> for VcardText<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardText(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}

impl<'v> Codec<'v> for VcardTextList<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardTextList(node.decode_at(0))
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        VcardValueNode {
            escaper,
            components: vec![encode_component(&self.0, escaper)],
        }
    }
}
