//! # NICKNAME
//!
//! The `NICKNAME` property: the familiar or informal name(s) of the
//! object, decoded as a comma-separated `VcardTextList` (RFC 6350 6.2.3).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `NICKNAME` property marker.
pub struct NICKNAME;

impl VcardPropSpec for NICKNAME {
    const KIND: VcardPropKind = VcardPropKind::Nickname;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0, VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::TextList]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Language,
            VcardParamKind::AltId,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Value,
        ]
    }
}
