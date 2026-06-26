//! The vCard version.

/// The VERSION property name.
pub const VERSION: &str = "VERSION";

/// The vCard version (the VERSION property): it governs grammar, so it is kept
/// apart from the content properties rather than as one of them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VcardVersion {
    /// vCard 2.1.
    V2_1,
    /// vCard 3.0 (RFC 2426).
    V3_0,
    /// vCard 4.0 (RFC 6350); the default.
    #[default]
    V4_0,
}

impl VcardVersion {
    /// The version named by a VERSION value, if it is one we support.
    pub fn from_value(value: &str) -> Option<Self> {
        match value {
            "2.1" => Some(Self::V2_1),
            "3.0" => Some(Self::V3_0),
            "4.0" => Some(Self::V4_0),
            _ => None,
        }
    }

    /// The wire value for this version.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::V2_1 => "2.1",
            Self::V3_0 => "3.0",
            Self::V4_0 => "4.0",
        }
    }
}
