//! # ORG value codec (RFC 6350 6.6.4)
//!
//! [`VcardCodec`] for the organization value: one or more `;`-separated units.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::org::VcardOrg,
};

impl<'v> VcardCodec<'v> for VcardOrg<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardOrg(node.decode_components().collect())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardOrg<'_> {
    /// Write each unit as its own component, holding one value.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        for unit in &self.0 {
            out.component(&[&**unit]);
        }
    }
}
