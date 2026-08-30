//! # KEY
//!
//! The `KEY` property: a public key or certificate.
//!
//! Its value shape is version-specific: a `data:` URI in 4.0, inline base64
//! or a URI reference in 2.1 / 3.0.
//!
//! See RFC 6350 6.8.1.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
    version::VcardVersion,
};

/// The `KEY` property marker.
pub struct KEY;

impl VcardPropSpec for KEY {
    const KIND: VcardPropKind = VcardPropKind::Key;

    fn allowed_values(version: VcardVersion) -> &'static [VcardValueKind] {
        match version {
            VcardVersion::V4_0 => &[VcardValueKind::Uri],
            _ => &[VcardValueKind::Binary, VcardValueKind::Uri],
        }
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::AltId,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::Value,
        ]
    }
}
