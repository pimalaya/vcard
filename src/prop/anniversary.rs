//! # ANNIVERSARY
//!
//! The `ANNIVERSARY` property: the contact's anniversary, decoded as a
//! date-and-or-time value.
//!
//! See RFC 6350 6.2.6.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `ANNIVERSARY` property marker.
pub struct ANNIVERSARY;

impl VcardPropSpec for ANNIVERSARY {
    const KIND: VcardPropKind = VcardPropKind::Anniversary;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::DateAndOrTime, VcardValueKind::Text]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::AltId,
            VcardParamKind::CalScale,
            VcardParamKind::Value,
        ]
    }
}
