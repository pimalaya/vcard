//! # CLIENTPIDMAP value codec (RFC 6350 6.7.7)
//!
//! [`Codec`] for the client-PID map: a source id and its URI.

use alloc::vec;

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper},
        value::VcardValueNode,
    },
    value::client_pid_map::VcardClientPidMap,
};

impl<'v> Codec<'v> for VcardClientPidMap<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardClientPidMap {
            id: node.decode_scalar_at(0),
            uri: node.decode_scalar_at(1),
        }
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        VcardValueNode {
            escaper,
            components: vec![
                encode_component(&[self.id.as_ref()], escaper),
                encode_component(&[self.uri.as_ref()], escaper),
            ],
        }
    }
}
