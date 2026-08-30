//! # GRAMGENDER
//!
//! The `GRAMGENDER` property: the grammatical gender to address the
//! contact by (e.g. `feminine`, `neuter`), decoded as text.
//!
//! See RFC 9554.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `GRAMGENDER` property marker.
pub struct GRAMGENDER;

impl VcardPropSpec for GRAMGENDER {
    const KIND: VcardPropKind = VcardPropKind::GramGender;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
