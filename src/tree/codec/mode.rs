//! # Escaping mode
//!
//! The one place the codec consults the card version: value escaping (RFC 6350
//! 3.4) differs between vCard 2.1 and the later versions, and parameter
//! encoding (RFC 6868) exists only from 4.0.
//!
//! A value node and a parameter node each carry a [`VcardEscaper`] telling the
//! sibling [`escape`](crate::tree::codec::escape) and
//! [`unescape`](crate::tree::codec::unescape) codecs which rules to apply.

use crate::version::VcardVersion;

/// The escaping rules to apply, selected by the card version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VcardEscaper {
    /// vCard 2.1 (versitcard): only `;` is escaped (`\;`) in a value; a
    /// backslash before anything else is literal.
    V2_1,
    /// vCard 3.0 (RFC 2426): the value escapes `\\`, `\,`, `\;` and `\n`.
    V3_0,
    /// vCard 4.0 (RFC 6350): the same value escapes as 3.0, plus the RFC 6868
    /// parameter value encoding.
    #[default]
    V4_0,
}

impl VcardEscaper {
    /// The escaping rules a card of `version` uses.
    pub fn for_version(version: VcardVersion) -> Self {
        match version {
            VcardVersion::V2_1 => Self::V2_1,
            VcardVersion::V3_0 => Self::V3_0,
            VcardVersion::V4_0 => Self::V4_0,
        }
    }

    /// The escaping rules for a raw `VERSION` wire string (e.g. `"2.1"`).
    pub fn for_version_str(version: &str) -> Self {
        match version.parse() {
            Ok(version) => Self::for_version(version),
            Err(_) => Self::default(),
        }
    }

    /// Whether this version carries the RFC 6868 parameter value encoding,
    /// which updates RFC 6350 and so reaches vCard 4.0 alone: 2.1 and 3.0
    /// predate it, and a caret in one of their parameters is a literal caret.
    pub fn has_param_encoding(self) -> bool {
        matches!(self, Self::V4_0)
    }

    /// Whether this version wraps a parameter value carrying a delimiter in
    /// double quotes: RFC 2425 section 5.1 defines the `quoted-string` RFC
    /// 2426 inherits and RFC 6350 section 3.3 keeps, so 3.0 and 4.0 have it
    /// and 2.1, whose grammar has none, reads its double quote as content.
    pub fn has_param_quoting(self) -> bool {
        matches!(self, Self::V3_0 | Self::V4_0)
    }
}
