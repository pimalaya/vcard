//! # ORG value codec (RFC 6350 6.6.4)
//!
//! [`VcardCodec`] for the organization value: one or more `;`-separated units.

use crate::{
    tree::{
        codec::{VcardCodec, encode::encode_component, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::org::VcardOrg,
};

impl<'v> VcardCodec<'v> for VcardOrg<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardOrg(node.decode_components().collect())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueNode::from_components(
            self.0
                .iter()
                .map(|unit| encode_component(&[unit.as_ref()], escaper))
                .collect(),
            escaper,
        )
    }
}
