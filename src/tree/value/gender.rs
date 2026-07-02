//! # GENDER value codec (RFC 6350 6.2.7)
//!
//! [`Codec`] for the structured gender: a sex component and an identity
//! component.

use alloc::vec;

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper},
        value::VcardValueNode,
    },
    value::gender::VcardGender,
};

impl<'v> Codec<'v> for VcardGender<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardGender {
            sex: node.decode_scalar_at(0),
            identity: node.decode_scalar_at(1),
        }
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        VcardValueNode::from_components(
            vec![
                encode_component(&[self.sex.as_ref()], escaper),
                encode_component(&[self.identity.as_ref()], escaper),
            ],
            escaper,
        )
    }
}
