//! # UTC-offset value codec (RFC 6350 4.7)
//!
//! [`Codec`] for a UTC-offset value, a single scalar kept as its raw text.

use crate::{
    tree::{
        codec::{encode::scalar_node, mode::Escaper, value::Codec},
        value::VcardValueNode,
    },
    value::utc_offset::VcardUtcOffset,
};

impl<'v> Codec<'v> for VcardUtcOffset<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardUtcOffset(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
