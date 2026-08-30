//! # LABEL
//!
//! The `LABEL` property: a formatted delivery-address label, decoded as a
//! single text value.
//!
//! See RFC 2426 3.2.2 (vCard 2.1/3.0; a parameter in 4.0).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `LABEL` property marker.
pub struct LABEL;

impl VcardPropSpec for LABEL {
    const KIND: VcardPropKind = VcardPropKind::Label;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V2_1, VcardVersion::V3_0]
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
