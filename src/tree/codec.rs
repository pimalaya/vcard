//! # Codec
//!
//! The bytes-to-model bridge, in both directions and at both levels. The only
//! part of [`crate::tree`] that consults the card version.
//!
//! [`decode`] projects a raw syntax tree onto the decoded model and [`encode`]
//! projects it back; that is the structural level.
//!
//! Underneath, at the value-string level, [`escape`] and [`unescape`] apply and
//! resolve the RFC 6350 3.4 value escapes, keyed by the [`mode`]
//! `VcardEscaper`, and every value leaf the structural codecs touch runs
//! through them.
//!
//! A parameter value is a different alphabet, with no backslash escapes at all
//! (RFC 6350 3.3) and the RFC 6868 caret encoding instead, so it has its own
//! pair in the same two modules, keyed by the same `VcardEscaper`.
//!
//! Content transfer encodings (`QUOTED-PRINTABLE`, `BASE64`) and `CHARSET` are
//! never resolved here: the core transforms no content, leaving that to the
//! opt-in feature helpers.
//!
//! The per-value-type projection is the [`VcardCodec`] trait, implemented once
//! per value type under [`crate::tree::value`], mirroring the model's `value/`;
//! both the structural dispatch and the per-property lenses go through it.

use crate::{
    tree::{
        codec::{encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::{VcardValue, VcardValueUnknown},
};

pub mod decode;
pub mod encode;
pub mod escape;
pub mod mode;
pub mod unescape;

/// How a decoded value type projects to and from a syntax node.
///
/// `decode` reads it from a node, whose [`escaper`](VcardValueNode::escaper)
/// carries the mode; `encode` writes it back, escaping every leaf with the
/// given [`VcardEscaper`]. A decoded value is itself escaper-agnostic text.
pub trait VcardCodec<'v>: Sized {
    /// Decode the value from a syntax node.
    fn decode(node: &'v VcardValueNode<'_>) -> Self;

    /// Encode the value into a syntax node for the given escaping mode.
    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static>;
}

impl<'v> VcardCodec<'v> for VcardValue<'v> {
    /// Decode liberally as raw [`Unknown`](VcardValue::Unknown).
    ///
    /// No value kind is known at this level, that being the spec's job: the
    /// version-divergent lenses targeting `VcardValue` override the lens
    /// `decode` to resolve it, and this fallback is what the others inherit.
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardValue::Unknown(VcardValueUnknown::decode(node))
    }

    /// Encode by dispatching to the held value's own shape.
    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardValue<'_> {
    /// Write the held value's wire shape.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        match self {
            VcardValue::Text(v) => v.emit(out),
            VcardValue::TextList(v) => v.emit(out),
            VcardValue::Uri(v) => v.emit(out),
            VcardValue::DateAndOrTime(v) => v.emit(out),
            VcardValue::Timestamp(v) => v.emit(out),
            VcardValue::LanguageTag(v) => v.emit(out),
            VcardValue::UtcOffset(v) => v.emit(out),
            VcardValue::N(v) => v.emit(out),
            VcardValue::Adr(v) => v.emit(out),
            VcardValue::Binary(v) => v.emit(out),
            VcardValue::Gender(v) => v.emit(out),
            VcardValue::Geo(v) => v.emit(out),
            VcardValue::Org(v) => v.emit(out),
            VcardValue::ClientPidMap(v) => v.emit(out),
            VcardValue::Unknown(v) => v.emit(out),
        }
    }
}
