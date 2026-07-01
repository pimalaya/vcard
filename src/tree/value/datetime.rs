//! # Date-and-or-time value codec (RFC 6350 4.3)
//!
//! [`Codec`] for a date-and-or-time value (4.3.4) and a timestamp (4.3.5), each
//! a single scalar kept as its raw text.

use crate::{
    tree::{
        codec::{Codec, encode::scalar_node, mode::Escaper},
        value::VcardValueNode,
    },
    value::datetime::{VcardDateAndOrTime, VcardTimestamp},
};

impl<'v> Codec<'v> for VcardDateAndOrTime<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardDateAndOrTime(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}

impl<'v> Codec<'v> for VcardTimestamp<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardTimestamp(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
