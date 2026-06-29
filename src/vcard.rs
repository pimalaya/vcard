//! # Card envelope vocabulary
//!
//! The wire names that frame every card, shared by all versions: [`VCARD`] (the
//! object name) and [`VCARD_BEGIN`] / [`VCARD_END`] (the `BEGIN:VCARD` /
//! `END:VCARD` delimiters). Pure name vocabulary with a single source of truth;
//! the decoded card and the per-version syntax trees both refer here.

/// The vCard object name, the value of the `BEGIN` and `END` lines.
pub const VCARD: &str = "VCARD";
/// The name of the line that opens a card.
pub const VCARD_BEGIN: &str = "BEGIN";
/// The name of the line that closes a card.
pub const VCARD_END: &str = "END";
