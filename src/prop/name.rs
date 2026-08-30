//! # NAME
//!
//! The `NAME` property: the displayable name of the source the card
//! describes, a vCard 3.0 property removed in 4.0. The lens decodes the value
//! as a single `VcardText` (RFC 2426 3.1.5).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `NAME` property marker.
pub struct NAME;

impl VcardPropSpec for NAME {
    const KIND: VcardPropKind = VcardPropKind::Name;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}
