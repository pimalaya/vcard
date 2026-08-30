//! # CLASS
//!
//! The `CLASS` property: the access classification of the card, decoded as
//! a single text value.
//!
//! See RFC 2426 3.7.1 (vCard 3.0; removed in 4.0).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `CLASS` property marker.
pub struct CLASS;

impl VcardPropSpec for CLASS {
    const KIND: VcardPropKind = VcardPropKind::Class;

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
