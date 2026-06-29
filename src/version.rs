//! # Version
//!
//! The card version value and its name vocabulary.
//!
//! [`VcardVersion`] is the decoded `VERSION` line: a known 2.1 / 3.0 / 4.0
//! value, or `Unknown` for anything else. The version sits apart from the other
//! properties because the syntax tree treats it as a required, fixed part of the
//! card envelope rather than a free property. Shared by every version module;
//! pure model, no syntax dependency. The per-version wire-string consts
//! (`VCARD_VERSION_21` and friends) live in their own version modules.

use alloc::borrow::Cow;

pub const VCARD_VERSION: &str = "VERSION";

/// The card version: a known value, or `Unknown` for anything else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardVersion<'a> {
    /// vCard 2.1.
    V21,
    /// vCard 3.0.
    V30,
    /// vCard 4.0.
    V40,
    /// Any version the model does not recognise.
    Unknown(Cow<'a, str>),
}

impl VcardVersion<'_> {
    /// The version's wire string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::V21 => "2.1",
            Self::V30 => "3.0",
            Self::V40 => "4.0",
            Self::Unknown(version) => version,
        }
    }
}

impl<'a> From<Cow<'a, str>> for VcardVersion<'a> {
    fn from(version: Cow<'a, str>) -> Self {
        match version.as_ref() {
            "2.1" => Self::V21,
            "3.0" => Self::V30,
            "4.0" => Self::V40,
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

    use crate::version::VcardVersion;

    #[test]
    fn maps_known_wire_strings_both_ways() {
        assert_eq!(VcardVersion::from("4.0"), VcardVersion::V40);
        assert_eq!(VcardVersion::from("2.1"), VcardVersion::V21);
        assert_eq!(VcardVersion::V30.as_str(), "3.0");
    }

    #[test]
    fn keeps_unknown_versions() {
        let version = VcardVersion::from("5.0");
        assert_eq!(version, VcardVersion::Unknown(Cow::Borrowed("5.0")));
        assert_eq!(version.as_str(), "5.0");
    }
}
