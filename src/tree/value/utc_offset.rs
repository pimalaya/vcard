//! # UTC-offset value codec (RFC 6350 4.7)
//!
//! [`VcardCodec`] for a UTC-offset value, a single scalar kept as its raw text.

use crate::{
    tree::{
        codec::{VcardCodec, encode::scalar_node, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::utc_offset::VcardUtcOffset,
};

impl<'v> VcardCodec<'v> for VcardUtcOffset<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardUtcOffset(node.decode())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
