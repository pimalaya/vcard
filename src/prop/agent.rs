//! # AGENT
//!
//! The `AGENT` property: another entity acting as the contact's agent,
//! kept as opaque text and decoded as a single text value.
//!
//! See RFC 2426 3.5.4 (vCard 3.0; removed in 4.0).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `AGENT` property marker.
pub struct AGENT;

impl VcardPropSpec for AGENT {
    const KIND: VcardPropKind = VcardPropKind::Agent;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V2_1, VcardVersion::V3_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Language,
            VcardParamKind::Encoding,
            VcardParamKind::Charset,
            VcardParamKind::Value,
        ]
    }
}
