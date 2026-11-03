//! # CLIENTPIDMAP value codec (RFC 6350 6.7.7)
//!
//! [`VcardCodec`] for the client-PID map: a source id and its URI.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
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
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardClientPidMap<'_> {
    /// Write the id and the URI as a component each.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&[&*self.id]);
        out.component(&[&*self.uri]);
    }
}
