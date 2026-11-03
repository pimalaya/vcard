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
        let mut lists = node.decode_component_lists();
        let mut next = || lists.next().unwrap_or_default();

        VcardN {
            family: next(),
            given: next(),
            additional: next(),
            prefixes: next(),
            suffixes: next(),
        }
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
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
