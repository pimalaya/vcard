//! # Language-tag value codec (RFC 6350 4.8)
//!
//! [`VcardCodec`] for a language-tag value, a single scalar kept as its raw
//! text.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::language::VcardLanguageTag,
};

impl<'v> VcardCodec<'v> for VcardLanguageTag<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardLanguageTag(node.decode())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardLanguageTag<'_> {
    /// Write the tag as one component holding its text.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
