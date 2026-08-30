//! # MAILER
//!
//! The `MAILER` property: the identifier of the contact's mail client,
//! decoded as a single text value.
//!
//! See RFC 2426 3.3.2 (vCard 2.1/3.0; removed in 4.0).

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    version::VcardVersion,
};

/// The `MAILER` property marker.
pub struct MAILER;

impl VcardPropSpec for MAILER {
    const KIND: VcardPropKind = VcardPropKind::Mailer;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V2_1, VcardVersion::V3_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::Encoding,
            VcardParamKind::Charset,
            VcardParamKind::Value,
        ]
    }
}
