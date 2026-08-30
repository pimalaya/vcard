//! # KIND
//!
//! The `KIND` property: the kind of entity the card represents, such as an
//! individual or an organization, decoded as a text value.
//!
//! See RFC 6350 6.1.4.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `KIND` property marker.
pub struct KIND;

impl VcardPropSpec for KIND {
    const KIND: VcardPropKind = VcardPropKind::Kind;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}
