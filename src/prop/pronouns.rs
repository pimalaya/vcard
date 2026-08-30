//! # PRONOUNS
//!
//! The `PRONOUNS` property: pronouns to refer to the contact by,
//! decoded as text.
//!
//! See RFC 9554.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `PRONOUNS` property marker.
pub struct PRONOUNS;

impl VcardPropSpec for PRONOUNS {
    const KIND: VcardPropKind = VcardPropKind::Pronouns;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
