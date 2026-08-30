//! # LOGO
//!
//! The `LOGO` property: the graphic logo of an organization.
//!
//! Its value shape is version-specific: a `data:` URI in 4.0, inline base64
//! or a URI reference in 2.1 / 3.0.
//!
//! See RFC 6350 6.6.3.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `LOGO` property marker.
pub struct LOGO;

impl VcardPropSpec for LOGO {
    const KIND: VcardPropKind = VcardPropKind::Logo;

    fn allowed_values(version: VcardVersion) -> &'static [VcardValueKind] {
        match version {
            VcardVersion::V4_0 => &[VcardValueKind::Uri],
            _ => &[VcardValueKind::Binary, VcardValueKind::Uri],
        }
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
