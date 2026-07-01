//! # Escaping mode
//!
//! The one place the codec consults the card version: value escaping (RFC 6350
//! 3.4) differs between vCard 2.1 and the later versions, so a value node
//! carries an [`Escaper`] telling the sibling
//! [`escape`](crate::tree::codec::escape) and
//! [`unescape`](crate::tree::codec::unescape) codecs which rules to apply.

use crate::version::VcardVersion;

/// The value-escaping rules to apply, selected by the card version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Escaper {
    /// vCard 2.1: only `;` is escaped (`\;`); a backslash before anything else
    /// is literal.
    V2_1,
    /// vCard 3.0 / 4.0 (RFC 2426 / 6350): `\\`, `\,`, `\;` and `\n`.
    #[default]
    Modern,
}

impl Escaper {
    /// The escaping rules a card of `version` uses.
    pub fn for_version(version: VcardVersion) -> Self {
        match version {
            VcardVersion::V2_1 => Self::V2_1,
            _ => Self::Modern,
        }
    }

    /// The escaping rules for a raw `VERSION` wire string (e.g. `"2.1"`).
    pub fn for_version_str(version: &str) -> Self {
        match version.parse() {
            Ok(VcardVersion::V2_1) => Self::V2_1,
            _ => Self::Modern,
        }
    }
}
