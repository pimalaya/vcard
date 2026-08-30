//! # XML
//!
//! The `XML` property: extended XML-encoded vCard data that fits no other
//! property, decoded as a single `VcardText` (RFC 6350 6.1.5).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `XML` property marker.
pub struct XML;

impl VcardPropSpec for XML {
    const KIND: VcardPropKind = VcardPropKind::Xml;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::AltId, VcardParamKind::Value]
    }
}
