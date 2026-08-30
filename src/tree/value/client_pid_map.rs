//! # CLIENTPIDMAP value codec (RFC 6350 6.7.7)
//!
//! [`VcardCodec`] for the client-PID map: a source id and its URI.

use alloc::vec;

use crate::{
    tree::{
        codec::{VcardCodec, encode::encode_component, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::client_pid_map::VcardClientPidMap,
};

impl<'v> VcardCodec<'v> for VcardClientPidMap<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardClientPidMap {
            id: node.decode_component(0),
            uri: node.decode_component(1),
        }
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueNode::from_components(
            vec![
                encode_component(&[self.id.as_ref()], escaper),
                encode_component(&[self.uri.as_ref()], escaper),
            ],
            escaper,
        )
    }
}
