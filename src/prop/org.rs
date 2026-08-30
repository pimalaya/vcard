//! # ORG
//!
//! The `ORG` (organization) property: the organizational name and units the
//! object belongs to, as a `;`-ordered list.
//!
//! See RFC 6350 6.6.4.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `ORG` property marker.
pub struct ORG;

impl VcardPropSpec for ORG {
    const KIND: VcardPropKind = VcardPropKind::Org;

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Org]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::SortAs,
            VcardParamKind::Language,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::Type,
            VcardParamKind::Value,
        ]
    }
}
