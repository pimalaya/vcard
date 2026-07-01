//! # Value codec
//!
//! The [`Codec`] trait: how a decoded value type reads itself from a
//! [`VcardValueNode`] and writes itself back. One impl per value type lives in a
//! submodule here, mirroring the model's `value/`. Both the structural
//! [`decode`](crate::tree::codec::decode) / [`encode`](crate::tree::codec::encode)
//! dispatch and the per-property lenses go through it, so each value's codec is
//! written exactly once.

use crate::{
    tree::{codec::mode::Escaper, value::VcardValueNode},
    value::{VcardUnknownValue, VcardValue},
};

pub mod adr;
pub mod binary;
pub mod client_pid_map;
pub mod datetime;
pub mod gender;
pub mod geo;
pub mod language;
pub mod n;
pub mod org;
pub mod text;
pub mod unknown;
pub mod uri;
pub mod utc_offset;

/// How a decoded value type projects to and from a syntax node: `decode` reads
/// it from a node (its [`escaper`](VcardValueNode::escaper) carries the mode),
/// `encode` writes it back, escaping every leaf with the given [`Escaper`] and
/// stamping it on the node. The escaper is symmetric across the two directions:
/// decode reads it off the incoming node, encode receives the target mode and
/// applies it (the decoded value itself is escaper-agnostic clean text).
pub trait Codec<'v>: Sized {
    /// Decode the value from a syntax node.
    fn decode(node: &'v VcardValueNode<'_>) -> Self;

    /// Encode the value into a syntax node for the given escaping mode.
    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static>;
}

impl<'v> Codec<'v> for VcardValue<'v> {
    /// Decode liberally as raw [`Unknown`](VcardValue::Unknown): no value kind is
    /// known at this level (that is the spec's job), so the version-divergent
    /// lenses whose target is `VcardValue` override the lens `decode` to resolve
    /// the real kind; this fallback is what the others inherit.
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardValue::Unknown(VcardUnknownValue::decode(node))
    }

    /// Encode by dispatching to the held value's own codec.
    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
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
