//! # TEL
//!
//! The `TEL` (telephone) property: a telephone number for the object,
//! decoded as a single `VcardText` (RFC 6350 6.4.1).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `TEL` property marker.
pub struct TEL;

impl VcardPropSpec for TEL {
    const KIND: VcardPropKind = VcardPropKind::Tel;

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Text, VcardValueKind::Uri]
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
