//! # Unknown value codec
//!
//! [`Codec`] for a value the model does not decode: its raw components are kept
//! (unescaped on read, re-escaped on write) so anything round-trips.

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper},
        value::VcardValueNode,
    },
    value::VcardUnknownValue,
};

impl<'v> Codec<'v> for VcardUnknownValue<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardUnknownValue {
            components: (0..node.component_count())
                .map(|i| node.decode_at(i))
                .collect(),
        }
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        VcardValueNode::from_components(
            self.components
                .iter()
                .map(|component| encode_component(component, escaper))
                .collect(),
            escaper,
        )
    }
}
