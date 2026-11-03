//! # N value codec (RFC 6350 6.2.2)
//!
//! [`VcardCodec`] for the structured name: five `;`-separated components.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::n::VcardN,
};

impl<'v> VcardCodec<'v> for VcardN<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        let mut n = VcardN::default();
        n.decode_into(node);
        n
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl<'v> VcardN<'v> {
    /// Decode `node` into this name, whose components are empty, each
    /// filled where it lies.
    pub(crate) fn decode_into(&mut self, node: &'v VcardValueNode<'_>) {
        node.decode_component_lists_into([
            &mut self.family,
            &mut self.given,
            &mut self.additional,
            &mut self.prefixes,
            &mut self.suffixes,
        ]);
    }
}

impl VcardN<'_> {
    /// Write the five components, empty ones included.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&self.family);
        out.component(&self.given);
        out.component(&self.additional);
        out.component(&self.prefixes);
        out.component(&self.suffixes);
    }
}
