//! # SOCIALPROFILE
//!
//! The `SOCIALPROFILE` property: a social-media profile, a URI by default or
//! a username when `VALUE=text` is declared.
//!
//! See RFC 9554.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `SOCIALPROFILE` property marker.
pub struct SOCIALPROFILE;

impl VcardPropSpec for SOCIALPROFILE {
    const KIND: VcardPropKind = VcardPropKind::SocialProfile;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Uri, VcardValueKind::Text]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::ServiceType,
            VcardParamKind::Username,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
