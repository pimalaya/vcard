//! # Date-and-or-time value codec (RFC 6350 4.3)
//!
//! [`VcardCodec`] for a date-and-or-time value (4.3.4) and a timestamp (4.3.5),
//! each a single scalar kept as its raw text.

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::datetime::{VcardDateAndOrTime, VcardTimestamp},
};

impl<'v> VcardCodec<'v> for VcardDateAndOrTime<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardDateAndOrTime(node.decode())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardDateAndOrTime<'_> {
    /// Write the value as one component holding its text.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}

impl<'v> VcardCodec<'v> for VcardTimestamp<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardTimestamp(node.decode())
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardTimestamp<'_> {
    /// Write the value as one component holding its text.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        out.component(&[&*self.0]);
    }
}
