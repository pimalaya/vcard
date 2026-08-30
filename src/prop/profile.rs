//! # PROFILE
//!
//! The `PROFILE` property: a vCard 3.0 declaration (value `VCARD`) stating
//! the card conforms to the vCard profile, removed in 4.0. The lens decodes it
//! as a single `VcardText` (RFC 2426 3.1.4).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `PROFILE` property marker.
pub struct PROFILE;

impl VcardPropSpec for PROFILE {
    const KIND: VcardPropKind = VcardPropKind::Profile;

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
