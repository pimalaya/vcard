//! # UID
//!
//! The `UID` property: a URI that globally and persistently identifies the
//! object the card represents, decoded as a `VcardUri` (RFC 6350 6.7.6).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `UID` property marker.
pub struct UID;

impl VcardPropSpec for UID {
    const KIND: VcardPropKind = VcardPropKind::Uid;

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Uri, VcardValueKind::Text]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}
