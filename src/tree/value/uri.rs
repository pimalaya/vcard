//! # URI value codec (RFC 6350 4.2)
//!
//! [`VcardCodec`] for a URI value. RFC 6350 section 4.2 gives a URI no
//! escaping and no structure, so its `;` and `,` are part of the reference:
//! the whole value is read, and written back exactly as it is held.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::uri::VcardUri,
};

impl<'v> VcardCodec<'v> for VcardUri<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardUri(node.decode())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardUri<'_> {
    /// Write the URI exactly as it is held.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.verbatim(&self.0);
    }
}
