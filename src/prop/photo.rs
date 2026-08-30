//! # PHOTO
//!
//! The `PHOTO` property: an image of the object.
//!
//! Its value shape is version-specific: a `data:` URI in 4.0, inline base64
//! or a URI reference in 2.1 / 3.0.
//!
//! See RFC 6350 6.2.4.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `PHOTO` property marker.
pub struct PHOTO;

impl VcardPropSpec for PHOTO {
    const KIND: VcardPropKind = VcardPropKind::Photo;

    fn allowed_values(version: VcardVersion) -> &'static [VcardValueKind] {
        match version {
            VcardVersion::V4_0 => &[VcardValueKind::Uri],
            _ => &[VcardValueKind::Binary, VcardValueKind::Uri],
        }
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::AltId,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::Pref,
            VcardParamKind::Pid,
            VcardParamKind::Value,
        ]
    }
}
