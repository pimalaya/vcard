//! # NOTE
//!
//! The `NOTE` property: supplemental free-form commentary about the
//! object, decoded as a single `VcardText` (RFC 6350 6.7.2).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `NOTE` property marker.
pub struct NOTE;

impl VcardPropSpec for NOTE {
    const KIND: VcardPropKind = VcardPropKind::Note;

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
