//! # CATEGORIES
//!
//! The `CATEGORIES` property: the tags the contact belongs to, decoded as
//! a comma-separated text list.
//!
//! See RFC 6350 6.7.1.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `CATEGORIES` property marker.
pub struct CATEGORIES;

impl VcardPropSpec for CATEGORIES {
    const KIND: VcardPropKind = VcardPropKind::Categories;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0, VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::TextList]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
