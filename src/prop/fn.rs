//! # FN
//!
//! The `FN` (formatted name) property: the contact's display name, decoded
//! as a single text value.
//!
//! See RFC 6350 6.2.1.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `FN` property marker.
pub struct FN;

impl VcardPropSpec for FN {
    const KIND: VcardPropKind = VcardPropKind::Fn;

    fn cardinality(version: VcardVersion) -> VcardPropCardinality {
        match version {
            VcardVersion::V2_1 => VcardPropCardinality::AtMostOne,
            _ => VcardPropCardinality::OneOrMore,
        }
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Language,
            VcardParamKind::AltId,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Value,
        ]
    }
}
