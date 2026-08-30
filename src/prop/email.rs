//! # EMAIL
//!
//! The `EMAIL` property: an email address for the contact, decoded as a
//! text value.
//!
//! See RFC 6350 6.4.2.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `EMAIL` property marker.
pub struct EMAIL;

impl VcardPropSpec for EMAIL {
    const KIND: VcardPropKind = VcardPropKind::Email;

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
