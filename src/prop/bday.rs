//! # BDAY
//!
//! The `BDAY` (birthday) property: the contact's birth date, decoded as a
//! date-and-or-time value.
//!
//! See RFC 6350 6.2.5.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `BDAY` property marker.
pub struct BDAY;

impl VcardPropSpec for BDAY {
    const KIND: VcardPropKind = VcardPropKind::Bday;

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
            VcardParamKind::Language,
            VcardParamKind::Value,
        ]
    }
}
