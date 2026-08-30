//! # RELATED
//!
//! The `RELATED` property: a relationship to another entity, qualified by
//! the `TYPE` parameter. The lens decodes the value as a `VcardUri`
//! (RFC 6350 6.6.6).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `RELATED` property marker.
pub struct RELATED;

impl VcardPropSpec for RELATED {
    const KIND: VcardPropKind = VcardPropKind::Related;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Uri, VcardValueKind::Text]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::Language,
            VcardParamKind::MediaType,
            VcardParamKind::Value,
        ]
    }
}
