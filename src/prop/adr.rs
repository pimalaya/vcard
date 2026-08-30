//! # ADR
//!
//! The `ADR` (structured address) property: the delivery address of the
//! object the card represents, as the seven RFC 6350 components and the
//! eleven RFC 9554 extensions.
//!
//! See RFC 6350 6.3.1 and RFC 9554.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `ADR` property marker.
pub struct ADR;

impl VcardPropSpec for ADR {
    const KIND: VcardPropKind = VcardPropKind::Adr;

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Adr]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Label,
            VcardParamKind::Language,
            VcardParamKind::Phonetic,
            VcardParamKind::Script,
            VcardParamKind::Geo,
            VcardParamKind::Tz,
            VcardParamKind::AltId,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::Value,
        ]
    }
}
