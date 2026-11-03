//! # GENDER value codec (RFC 6350 6.2.7)
//!
//! [`VcardCodec`] for the structured gender: a sex component and an identity
//! component.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
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
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardGender<'_> {
    /// Write the sex and the identity as a component each.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&[&*self.sex]);
        out.component(&[&*self.identity]);
    }
}
