//! # SOUND
//!
//! The `SOUND` property: a digital sound associated with the object, such as
//! a name pronunciation.
//!
//! Its value shape is version-specific: a `data:` URI in 4.0, inline base64
//! or a URI reference in 2.1 / 3.0.
//!
//! See RFC 6350 6.7.5.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `SOUND` property marker.
pub struct SOUND;

impl VcardPropSpec for SOUND {
    const KIND: VcardPropKind = VcardPropKind::Sound;

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
