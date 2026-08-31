//! # GENDER
//!
//! The `GENDER` property, pairing a sex code with a free-text identity.
//!
//! See RFC 6350 6.2.7.

use alloc::string::{String, ToString};

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::VcardPropSpec},
    value::{VcardValue, VcardValueKind},
    version::VcardVersion,
};

/// The `GENDER` property marker.
pub struct GENDER;

impl VcardPropSpec for GENDER {
    const KIND: VcardPropKind = VcardPropKind::Gender;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Gender]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }

    /// RFC 6350 6.2.7 closes the sex component:
    ///
    /// ```abnf
    /// sex = "" / "M" / "F" / "O" / "N" / "U"
    /// ```
    ///
    /// No `iana-token` and no `x-name`, which is what makes it different
    /// from every other vocabulary in the format. Anything else belongs in
    /// the identity component, which is free text: the RFC's own examples
    /// are `GENDER:O;intersex` and `GENDER:;it's complicated`.
    fn invalid_value(value: &VcardValue<'_>, _version: VcardVersion) -> Option<String> {
        let VcardValue::Gender(gender) = value else {
            return None;
        };

        let sex = gender.sex.as_ref();

        SEXES
            .iter()
            .all(|allowed| !sex.eq_ignore_ascii_case(allowed))
            .then(|| sex.to_string())
    }
}

/// The sex codes RFC 6350 6.2.7 defines, the empty one included.
const SEXES: &[&str] = &["", "M", "F", "O", "N", "U"];
