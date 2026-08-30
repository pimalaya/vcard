//! # GENDER value codec (RFC 6350 6.2.7)
//!
//! [`VcardCodec`] for the structured gender: a sex component and an identity
//! component.

use alloc::vec;

use crate::{
    tree::{
        codec::{VcardCodec, encode::encode_component, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::gender::VcardGender,
};

impl<'v> VcardCodec<'v> for VcardGender<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardGender {
            sex: node.decode_component(0),
            identity: node.decode_component(1),
        }
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueNode::from_components(
            vec![
                encode_component(&[self.sex.as_ref()], escaper),
                encode_component(&[self.identity.as_ref()], escaper),
            ],
            escaper,
        )
    }
}
