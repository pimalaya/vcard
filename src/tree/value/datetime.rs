//! # Date-and-or-time value codec (RFC 6350 4.3)
//!
//! [`VcardCodec`] for a date-and-or-time value (4.3.4) and a timestamp (4.3.5),
//! each a single scalar kept as its raw text.

use crate::{
    tree::{
        codec::{VcardCodec, encode::scalar_node, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::datetime::{VcardDateAndOrTime, VcardTimestamp},
};

impl<'v> VcardCodec<'v> for VcardDateAndOrTime<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardDateAndOrTime(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}

impl<'v> VcardCodec<'v> for VcardTimestamp<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardTimestamp(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
