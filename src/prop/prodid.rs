//! # PRODID
//!
//! The `PRODID` property: the identifier of the product that created the
//! card, decoded as a single `VcardText` (RFC 6350 6.7.3).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `PRODID` property marker.
pub struct PRODID;

impl VcardPropSpec for PRODID {
    const KIND: VcardPropKind = VcardPropKind::ProdId;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0, VcardVersion::V4_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}
