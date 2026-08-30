//! # TZ
//!
//! The `TZ` (time zone) property: the time zone of the object. The lens
//! decodes it as a single `VcardText` (its default form; the UTC-offset and URI
//! forms round-trip as text) (RFC 6350 6.5.1).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `TZ` property marker.
pub struct TZ;

impl VcardPropSpec for TZ {
    const KIND: VcardPropKind = VcardPropKind::Tz;

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::AltId,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::Value,
        ]
    }
}
