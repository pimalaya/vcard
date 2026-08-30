//! # SORT-STRING
//!
//! The `SORT-STRING` property: the string a vCard 3.0 application should
//! use when sorting the card, decoded as a single `VcardText`. Replaced by the
//! `SORT-AS` parameter in 4.0 (RFC 2426 3.3.4).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `SORT-STRING` property marker.
#[allow(non_camel_case_types)]
pub struct SORT_STRING;

impl VcardPropSpec for SORT_STRING {
    const KIND: VcardPropKind = VcardPropKind::SortString;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Language, VcardParamKind::Value]
    }
}
