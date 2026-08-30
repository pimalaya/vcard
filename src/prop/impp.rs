//! # IMPP
//!
//! The `IMPP` property: a URI for instant messaging and presence with the
//! contact, decoded as a URI.
//!
//! See RFC 6350 6.4.3.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `IMPP` property marker.
pub struct IMPP;

impl VcardPropSpec for IMPP {
    const KIND: VcardPropKind = VcardPropKind::Impp;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0, VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Uri]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::MediaType,
            VcardParamKind::Value,
        ]
    }
}
