//! # FBURL
//!
//! The `FBURL` property: the URL to the contact's free/busy information,
//! decoded as a URI.
//!
//! See RFC 6350 6.9.1.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `FBURL` property marker.
pub struct FBURL;

impl VcardPropSpec for FBURL {
    const KIND: VcardPropKind = VcardPropKind::FbUrl;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Uri]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
