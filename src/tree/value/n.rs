//! # N value codec (RFC 6350 6.2.2)
//!
//! [`Codec`] for the structured name: five `;`-separated components.

use alloc::vec;

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper},
        value::VcardValueNode,
    },
    value::n::VcardN,
};

impl<'v> Codec<'v> for VcardN<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardN {
            family: node.decode_at(0),
            given: node.decode_at(1),
            additional: node.decode_at(2),
            prefixes: node.decode_at(3),
            suffixes: node.decode_at(4),
        }
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        VcardValueNode::from_components(
            vec![
                encode_component(&self.family, escaper),
                encode_component(&self.given, escaper),
                encode_component(&self.additional, escaper),
                encode_component(&self.prefixes, escaper),
                encode_component(&self.suffixes, escaper),
            ],
            escaper,
        )
    }
}
