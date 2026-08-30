//! # REV
//!
//! The `REV` (revision) property: the timestamp of the card's latest
//! revision, decoded as a `VcardTimestamp` (RFC 6350 6.7.4).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `REV` property marker.
pub struct REV;

impl VcardPropSpec for REV {
    const KIND: VcardPropKind = VcardPropKind::Rev;

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Timestamp]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}
