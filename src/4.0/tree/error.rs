//! # Errors
//!
//! The parsing errors.
//!
//! [`VcardParseError`] is the single error type returned by
//! [`VcardCst::parse`](crate::v40::tree::cst::VcardCst::parse) and the line tokeniser
//! it drives. Each variant pinpoints one structural failure (a missing CRLF, a
//! missing colon, an absent or malformed envelope) and carries the offending
//! text for context. Parsing is the only fallible bridge in the crate; decoding,
//! encoding and serializing never fail, so this is the whole error surface.

use alloc::string::String;
use core::fmt;

/// An error raised while parsing vCard text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VcardParseError {
    /// A line carried no CR?LF separator.
    MissingCrlf(String),
    /// A content line carried no colon separating the name from the value.
    MissingPropertyColon(String),
    /// A card did not open with a BEGIN:VCARD line.
    ExpectedBegin(String),
    /// A card did not follow BEGIN with a VERSION line.
    ExpectedVersion(String),
    /// A VERSION value named a version the parser does not support.
    UnsupportedVersion(String),
    /// A card was left open by a missing END:VCARD line.
    MissingEnd(String),
    /// A single-card parse found no card, or more than one.
    ExpectedSingleCard,
}

impl fmt::Display for VcardParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCrlf(data) => {
                write!(f, "Content is missing a line separator: {data}")
            }
            Self::MissingPropertyColon(data) => {
                write!(f, "Content line is missing a value separator: {data}")
            }
            Self::ExpectedBegin(data) => {
                write!(f, "Card does not open with a BEGIN line: {data}")
            }
            Self::ExpectedVersion(data) => {
                write!(f, "Card does not follow BEGIN with a VERSION line: {data}")
            }
            Self::UnsupportedVersion(data) => {
                write!(f, "Card names an unsupported version: {data}")
            }
            Self::MissingEnd(data) => {
                write!(f, "Card is left open by a missing END line: {data}")
            }
            Self::ExpectedSingleCard => {
                write!(f, "Content does not hold exactly one card")
            }
        }
    }
}

impl core::error::Error for VcardParseError {}
