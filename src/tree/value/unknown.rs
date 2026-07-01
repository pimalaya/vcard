//! # Unknown value codec
//!
//! [`Codec`] for a value the model does not decode: its raw components are kept
//! (unescaped on read, re-escaped on write) so anything round-trips.

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper, unescape::unescape_with},
        value::VcardValueNode,
    },
    value::VcardUnknownValue,
};

impl<'v> Codec<'v> for VcardUnknownValue<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardUnknownValue {
            components: node
                .components
                .iter()
                .map(|component| {
                    component
                        .iter()
                        .map(|value| unescape_with(value.get(), node.escaper))
                        .collect()
                })
                .collect(),
        }
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        VcardValueNode {
            escaper,
            components: self
                .components
                .iter()
                .map(|component| encode_component(component, escaper))
                .collect(),
        }
    }
}
