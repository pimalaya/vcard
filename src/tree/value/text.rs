//! # Text value codec (RFC 6350 4.1)
//!
//! [`VcardCodec`] for a single text value and a comma-separated text list. A
//! single text value keeps its whole component, commas included, since a comma
//! separates items only in a list.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::text::{VcardText, VcardTextList},
};

impl<'v> VcardCodec<'v> for VcardText<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        // NOTE: a comma must be escaped inside a text value, so an unescaped
        // one is content rather than a separator; cutting the value there
        // would drop everything after it.
        VcardText(node.decode())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardText<'_> {
    /// Write the text as one component holding one value.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}

impl<'v> VcardCodec<'v> for VcardTextList<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardTextList(node.decode_list())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardTextList<'_> {
    /// Write the list as one component of its values.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&self.0);
    }
}
