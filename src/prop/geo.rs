//! # GEO
//!
//! The `GEO` property: a geographic position.
//!
//! Its value shape is version-specific: a `geo:` URI in 4.0, a `lat;long`
//! coordinate pair in 2.1 / 3.0.
//!
//! See RFC 6350 6.5.2.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `GEO` property marker.
pub struct GEO;

impl VcardPropSpec for GEO {
    const KIND: VcardPropKind = VcardPropKind::Geo;

    fn allowed_values(version: VcardVersion) -> &'static [VcardValueKind] {
        match version {
            VcardVersion::V4_0 => &[VcardValueKind::Uri],
            _ => &[VcardValueKind::Geo],
        }
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
