//! # LANG
//!
//! The `LANG` property: a language the contact may be contacted in,
//! decoded as an RFC 5646 language tag.
//!
//! See RFC 6350 6.4.4.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `LANG` property marker.
pub struct LANG;

impl VcardPropSpec for LANG {
    const KIND: VcardPropKind = VcardPropKind::Lang;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::LanguageTag]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
