//! # Binary value codec (vCard 2.1 / 3.0)
//!
//! [`VcardCodec`] for a 2.1 / 3.0 `PHOTO` / `LOGO` / `SOUND` / `KEY` value:
//! inline base64 kept verbatim. A URI reference is reached via `VALUE=uri`,
//! which the spec resolves to [`VcardUri`](crate::value::uri::VcardUri), not
//! here.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::binary::VcardBinary,
};

impl<'v> VcardCodec<'v> for VcardBinary<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardBinary::Base64(node.decode())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardBinary<'_> {
    /// Write the value, either form, as one component holding its text.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        let (VcardBinary::Uri(value) | VcardBinary::Base64(value)) = self;
        out.component(&[&**value]);
    }
}
