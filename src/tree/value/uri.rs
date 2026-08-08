//! # URI value codec (RFC 6350 4.2)
//!
//! [`VcardCodec`] for a URI value. A URI's comma is literal, not a list
//! separator, so the whole component is kept (not truncated at `,`).

use crate::{
    tree::{
        codec::{VcardCodec, encode::scalar_node, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::uri::VcardUri,
};

impl<'v> VcardCodec<'v> for VcardUri<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardUri(node.decode_joined_at(0))
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        scalar_node(&self.0, escaper)
    }
}
