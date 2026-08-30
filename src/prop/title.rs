//! # TITLE
//!
//! The `TITLE` (job title) property: the position or job of the object,
//! decoded as a single `VcardText` (RFC 6350 6.6.1).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `TITLE` property marker.
pub struct TITLE;

impl VcardPropSpec for TITLE {
    const KIND: VcardPropKind = VcardPropKind::Title;

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
