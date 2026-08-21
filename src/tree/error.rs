//! # Errors
//!
//! The parsing errors.
//!
//! [`VcardParseError`] is the single error type returned by `VcardCst::parse`
//! and the line tokeniser it drives, each variant pinpointing one structural
//! failure and carrying the offending text. Parsing is the only fallible bridge
//! in the crate: decoding, encoding and serializing never fail, so this is the
//! whole error surface.

use core::{error, fmt};

use alloc::string::String;

/// An error raised while parsing vCard text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VcardParseError {
    /// A line carried no CR?LF separator.
    MissingCrlf(String),
    /// A content line carried no colon separating the name from the value.
    MissingPropertyColon(String),
    /// A content line's name or parameters were not valid UTF-8; only a value
    /// may carry a foreign charset.
    NonUtf8Header(String),
    /// A card did not open with a BEGIN:VCARD line.
    ExpectedBegin(String),
    /// A card was left open by a missing END:VCARD line.
    MissingEnd(String),
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
            Self::NonUtf8Header(data) => {
                write!(
                    f,
                    "Content line name or parameters are not valid UTF-8: {data}"
                )
            }
            Self::ExpectedBegin(data) => {
                write!(f, "Card does not open with a BEGIN line: {data}")
            }
            Self::MissingEnd(data) => {
                write!(f, "Card is left open by a missing END line: {data}")
            }
        }
    }
}

impl error::Error for VcardParseError {}
