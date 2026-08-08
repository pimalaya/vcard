//! # Unknown value codec
//!
//! [`VcardCodec`] for a value the model does not decode: its raw components are
//! kept (unescaped on read, re-escaped on write) so anything round-trips.

use crate::{
    tree::{
        codec::{VcardCodec, encode::encode_component, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::VcardValueUnknown,
};

impl<'v> VcardCodec<'v> for VcardValueUnknown<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardValueUnknown {
            components: (0..node.component_count())
                .map(|i| node.decode_at(i))
                .collect(),
        }
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueNode::from_components(
            self.components
                .iter()
                .map(|component| encode_component(component, escaper))
                .collect(),
            escaper,
        )
    }
}
