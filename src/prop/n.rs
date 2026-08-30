//! # N
//!
//! The `N` (structured name) property: the components of the name of the
//! object the card represents.
//!
//! See RFC 6350 6.2.2.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `N` property marker.
pub struct N;

impl VcardPropSpec for N {
    const KIND: VcardPropKind = VcardPropKind::N;

    fn cardinality(version: VcardVersion) -> VcardPropCardinality {
        match version {
            VcardVersion::V4_0 => VcardPropCardinality::AtMostOne,
            _ => VcardPropCardinality::ExactlyOne,
        }
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::N]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::SortAs,
            VcardParamKind::Language,
            VcardParamKind::Phonetic,
            VcardParamKind::Script,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
