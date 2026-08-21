//! # Version
//!
//! The card version indicator.
//!
//! [`VcardVersion`] is the decoded `VERSION` line: one of the three defined
//! versions (2.1 / 3.0 / 4.0), an unrecognised or missing one normalising to
//! [`V4_0`](VcardVersion::V4_0) at decode time. It sits apart from the other
//! properties because the syntax tree, which is what preserves the raw
//! `VERSION` line byte for byte, treats it as part of the card envelope. Pure
//! model, no syntax dependency.

use core::{error, fmt, ops, str};

use alloc::string::{String, ToString};

/// Parse vCard version error.
#[derive(Debug)]
pub struct VcardVersionParseError(
    /// The vCard version that cannot be parsed.
    String,
);

impl fmt::Display for VcardVersionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse vCard version `{}`", self.0)
    }
}

impl error::Error for VcardVersionParseError {}

/// The vCard version: one of the three defined versions. An unrecognised or
/// missing version normalises to [`V4_0`](Self::V4_0) (see the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcardVersion {
    /// vCard 2.1 (versitcard).
    V2_1,
    /// vCard 3.0 (RFC 2426).
    V3_0,
    /// vCard 4.0 (RFC 6350).
    V4_0,
}

impl str::FromStr for VcardVersion {
    type Err = VcardVersionParseError;

    /// The defined version for a wire string (`2.1`, `3.0`, `4.0`).
    fn from_str(version: &str) -> Result<Self, Self::Err> {
        match version {
            "2.1" => Ok(Self::V2_1),
            "3.0" => Ok(Self::V3_0),
            "4.0" => Ok(Self::V4_0),
            _ => Err(VcardVersionParseError(version.to_string())),
        }
    }
}

impl ops::Deref for VcardVersion {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::V2_1 => "2.1",
            Self::V3_0 => "3.0",
            Self::V4_0 => "4.0",
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::version::VcardVersion;

    #[test]
    fn maps_known_wire_strings_both_ways() {
        assert_eq!("2.1".parse().ok(), Some(VcardVersion::V2_1));
        assert_eq!(VcardVersion::V3_0.to_string(), "3.0");
        assert_eq!(&*VcardVersion::V4_0, "4.0");
    }

    #[test]
    fn rejects_unknown_versions() {
        let error = "5.0".parse::<VcardVersion>().unwrap_err();
        assert!(error.to_string().contains("5.0"));
    }
}
