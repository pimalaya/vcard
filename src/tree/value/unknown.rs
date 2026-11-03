//! # Unknown value codec
//!
//! [`VcardCodec`] for a value the model does not decode: its raw components are
//! kept (unescaped on read, re-escaped on write) so anything round-trips.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::VcardValueUnknown,
};

impl<'v> VcardCodec<'v> for VcardValueUnknown<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        let mut unknown = VcardValueUnknown::default();
        node.decode_component_lists_onto(&mut unknown.components);
        unknown
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardValueUnknown<'_> {
    /// Write every component as it is held.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        for component in &self.components {
            out.component(component);
        }
    }
}
