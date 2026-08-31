//! # PROFILE
//!
//! The `PROFILE` property: a vCard 3.0 declaration (value `VCARD`) stating
//! the card conforms to the vCard profile, removed in 4.0. The lens decodes it
//! as a single `VcardText` (RFC 2426 3.1.4).

use alloc::string::{String, ToString};

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    value::VcardValue,
    version::VcardVersion,
};

/// The `PROFILE` property marker.
pub struct PROFILE;

impl VcardPropSpec for PROFILE {
    const KIND: VcardPropKind = VcardPropKind::Profile;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }

    /// RFC 2426 3.6.3: the value "MUST be the case-insensitive string
    /// `VCARD`". The property exists to say the card is a vCard, so it has
    /// exactly one thing it can say.
    fn invalid_value(value: &VcardValue<'_>, _version: VcardVersion) -> Option<String> {
        let VcardValue::Text(text) = value else {
            return None;
        };

        let value = text.0.as_ref();

        (!value.eq_ignore_ascii_case("VCARD")).then(|| value.to_string())
    }
}
