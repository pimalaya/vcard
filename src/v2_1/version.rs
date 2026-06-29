//! # Version
//!
//! The card version value and its name vocabulary.
//!
//! [`VcardVersion`] is the decoded `VERSION` line: a known 2.1 / 3.0 / 4.0
//! value, or `Unknown` for anything else, with the wire strings backed by the
//! `VCARD_VERSION_*` consts so they have a single source of truth in both
//! directions. The version sits apart from the other properties because the
//! syntax tree treats it as a required, fixed part of the card envelope rather
//! than a free property. Pure model, no dependency on [`crate::v2_1::tree`].

use alloc::borrow::Cow;

pub const VCARD_VERSION: &str = "VERSION";
pub const VCARD_VERSION_21: &str = "2.1";
pub const VCARD_VERSION_30: &str = "3.0";
pub const VCARD_VERSION_40: &str = "4.0";

/// The card version: a known value, or `Unknown` for anything else. The known
/// variants are backed by the `VCARD_VERSION_*` consts, so the wire strings
/// have a single source of truth in both directions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum VcardVersion<'a> {
    /// vCard 2.1.
    V2_1,
    /// vCard 3.0.
    V3_0,
    /// vCard 4.0.
    #[default]
    V4_0,
    /// Any version the model does not recognise.
    Unknown(Cow<'a, str>),
}

impl VcardVersion<'_> {
    /// The version's wire string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::V2_1 => VCARD_VERSION_21,
            Self::V3_0 => VCARD_VERSION_30,
            Self::V4_0 => VCARD_VERSION_40,
            Self::Unknown(version) => version,
        }
    }
}

impl<'a> From<Cow<'a, str>> for VcardVersion<'a> {
    fn from(version: Cow<'a, str>) -> Self {
        match version.as_ref() {
            VCARD_VERSION_21 => Self::V2_1,
            VCARD_VERSION_30 => Self::V3_0,
            VCARD_VERSION_40 => Self::V4_0,
            _ => Self::Unknown(version),
        }
    }
}

impl<'a> From<&'a str> for VcardVersion<'a> {
    fn from(version: &'a str) -> Self {
        Cow::Borrowed(version).into()
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::v2_1::version::VcardVersion;

    #[test]
    fn maps_known_wire_strings_both_ways() {
        assert_eq!(VcardVersion::from("4.0"), VcardVersion::V4_0);
        assert_eq!(VcardVersion::from("2.1"), VcardVersion::V2_1);
        assert_eq!(VcardVersion::V3_0.as_str(), "3.0");
    }

    #[test]
    fn keeps_unknown_versions() {
        let version = VcardVersion::from("5.0");
        assert_eq!(version, VcardVersion::Unknown(Cow::Borrowed("5.0")));
        assert_eq!(version.as_str(), "5.0");
    }
}
