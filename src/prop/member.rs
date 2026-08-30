//! # MEMBER
//!
//! The `MEMBER` property: a URI naming a member of the group the card
//! represents, meaningful only when `KIND` is `group`. The lens decodes the
//! value as a `VcardUri` (RFC 6350 6.6.5).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `MEMBER` property marker.
pub struct MEMBER;

impl VcardPropSpec for MEMBER {
    const KIND: VcardPropKind = VcardPropKind::Member;

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
            VcardParamKind::AltId,
            VcardParamKind::MediaType,
            VcardParamKind::Value,
        ]
    }
}
