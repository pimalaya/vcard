//! # ROLE
//!
//! The `ROLE` property: the function or part the object plays in its
//! organization, decoded as a single `VcardText` (RFC 6350 6.6.2).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `ROLE` property marker.
pub struct ROLE;

impl VcardPropSpec for ROLE {
    const KIND: VcardPropKind = VcardPropKind::Role;

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::Type,
            VcardParamKind::Value,
        ]
    }
}
