//! # CALADRURI
//!
//! The `CALADRURI` property: the URI to use when sending a scheduling
//! request to the contact, decoded as a URI.
//!
//! See RFC 6350 6.9.2.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `CALADRURI` property marker.
pub struct CALADRURI;

impl VcardPropSpec for CALADRURI {
    const KIND: VcardPropKind = VcardPropKind::CalAdrUri;

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
