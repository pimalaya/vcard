//! # Codec
//!
//! The bytes-to-model bridge, in both directions and at both levels. The only
//! part of [`crate::tree`] that consults the card version.
//!
//! [`decode`] projects a raw syntax tree onto the decoded model and [`encode`]
//! projects it back; that is the structural level. Underneath, at the
//! value-string level, [`escape`] and [`unescape`] apply and resolve the RFC
//! 6350 3.4 value escapes, keyed by the [`mode`] `VcardEscaper`, and every value
//! leaf the structural codecs touch runs through them. Content transfer
//! encodings (`QUOTED-PRINTABLE`, `BASE64`) and `CHARSET` are never resolved
//! here: the core transforms no content, leaving that to the opt-in feature
//! helpers.
//!
//! The per-value-type projection is the [`VcardCodec`] trait, implemented once
//! per value type under [`crate::tree::value`], mirroring the model's `value/`;
//! both the structural dispatch and the per-property lenses go through it.

use crate::{
    tree::{codec::mode::VcardEscaper, value::node::VcardValueNode},
    value::{VcardValue, VcardValueUnknown},
};

pub mod decode;
pub mod encode;
pub mod escape;
pub mod mode;
pub mod unescape;

/// How a decoded value type projects to and from a syntax node: `decode` reads
/// it from a node (its [`escaper`](VcardValueNode::escaper) carries the mode),
/// `encode` writes it back, escaping every leaf with the given [`VcardEscaper`]
/// and stamping it on the node. The escaper is symmetric across the two
/// directions: decode reads it off the incoming node, encode receives the
/// target mode and applies it (the decoded value itself is escaper-agnostic
/// clean text).
pub trait VcardCodec<'v>: Sized {
    /// Decode the value from a syntax node.
    fn decode(node: &'v VcardValueNode<'_>) -> Self;

    /// Encode the value into a syntax node for the given escaping mode.
    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static>;
}

impl<'v> VcardCodec<'v> for VcardValue<'v> {
    /// Decode liberally as raw [`Unknown`](VcardValue::Unknown): no value kind
    /// is known at this level (that is the spec's job), so the
    /// version-divergent lenses whose target is `VcardValue` override the lens
    /// `decode` to resolve the real kind; this fallback is what the others
    /// inherit.
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardValue::Unknown(VcardValueUnknown::decode(node))
    }

    /// Encode by dispatching to the held value's own codec.
    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        match self {
            VcardValue::Text(v) => v.encode(escaper),
            VcardValue::TextList(v) => v.encode(escaper),
            VcardValue::Uri(v) => v.encode(escaper),
            VcardValue::DateAndOrTime(v) => v.encode(escaper),
            VcardValue::Timestamp(v) => v.encode(escaper),
            VcardValue::LanguageTag(v) => v.encode(escaper),
            VcardValue::UtcOffset(v) => v.encode(escaper),
            VcardValue::N(v) => v.encode(escaper),
            VcardValue::Adr(v) => v.encode(escaper),
            VcardValue::Binary(v) => v.encode(escaper),
            VcardValue::Gender(v) => v.encode(escaper),
            VcardValue::Geo(v) => v.encode(escaper),
            VcardValue::Org(v) => v.encode(escaper),
            VcardValue::ClientPidMap(v) => v.encode(escaper),
            VcardValue::Unknown(v) => v.encode(escaper),
        }
    }
}
