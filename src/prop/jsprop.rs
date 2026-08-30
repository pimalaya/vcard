//! # JSPROP
//!
//! The `JSPROP` property: a JSContact property with no vCard
//! counterpart, preserved as JSON text at the position named by its `JSPTR`
//! parameter.
//!
//! See RFC 9555.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `JSPROP` property marker.
pub struct JSPROP;

impl VcardPropSpec for JSPROP {
    const KIND: VcardPropKind = VcardPropKind::JsProp;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Jsptr, VcardParamKind::Value]
    }
}
