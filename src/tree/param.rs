//! # Parameters (syntax side)
//!
//! The per-parameter lens contract ([`VcardParamLens`](lens::VcardParamLens))
//! and one hand-written module per RFC 6350 parameter, tying a wire name to its
//! decoded shape; the markers are the type-level keys for
//! [`VcardLine::param`](crate::tree::line::VcardLine::param). The raw
//! [`VcardParamNode`](node::VcardParamNode) they read and write is defined
//! alongside. The name dispatch for whole-line decoding lives in
//! [`crate::tree::codec::decode`].

pub mod altid;
pub mod calscale;
pub mod geo;
pub mod label;
pub mod language;
pub mod lens;
pub mod mediatype;
pub mod node;
pub mod pid;
pub mod pref;
pub mod sort_as;
pub mod r#type;
pub mod tz;
pub mod value;

use crate::param::VcardParamKind;

/// The default parameters a property may carry, used by the spec for the
/// uniform majority. Per-property sets refine this where a property allows more
/// or fewer.
pub(crate) const COMMON_PARAMS: &[VcardParamKind] = &[
    VcardParamKind::Value,
    VcardParamKind::Language,
    VcardParamKind::Pref,
    VcardParamKind::AltId,
    VcardParamKind::Pid,
    VcardParamKind::Type,
];
