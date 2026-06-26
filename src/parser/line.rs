use core::ops::Range;

use alloc::string::ToString;

use crate::error::VcardParseError;

/// The byte ranges that make up one content line within the input.
pub(crate) struct VcardLine {
    /// The property name and parameters, before the colon.
    pub(crate) prop: Range<usize>,
    /// The value, after the colon and before the line break.
    pub(crate) value: Range<usize>,
    /// The line break (CR?LF) terminating the line.
    pub(crate) crlf: Range<usize>,
}

impl VcardLine {
    /// Tokenise the line that begins at `start` into its name, value and
    /// line-break ranges, all absolute into `input`.
    pub(crate) fn parse(input: &str, start: usize) -> Result<Self, VcardParseError> {
        let bytes = input.as_bytes();

        let Some(lf) = memchr::memchr(b'\n', &bytes[start..]) else {
            return Err(VcardParseError::MissingCrlf(input[start..].to_string()));
        };

        let lf = start + lf;

        let crlf = match lf > start && bytes[lf - 1] == b'\r' {
            true => lf - 1..lf + 1,
            false => lf..lf + 1,
        };

        let Some(colon) = memchr::memchr(b':', &bytes[start..crlf.start]) else {
            let prop = input[start..crlf.start].to_string();
            return Err(VcardParseError::MissingPropertyColon(prop));
        };

        let colon = start + colon;

        Ok(Self {
            prop: start..colon,
            value: colon + 1..crlf.start,
            crlf,
        })
    }
}
