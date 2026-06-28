use alloc::string::ToString;

use crate::error::VcardParseError;

/// One tokenised content line: the head (name and parameters, before the
/// colon), the value (after the colon), and the line ending, all borrowed from
/// the source.
pub(crate) struct VcardLine<'a> {
    /// The property name and parameters, before the colon.
    pub(crate) head: &'a str,
    /// The value, after the colon and before the line ending.
    pub(crate) value: &'a str,
    /// The line ending that terminated the line (`\r\n` or `\n`).
    pub(crate) eol: &'a str,
}

impl<'a> VcardLine<'a> {
    /// Tokenise the line at the start of `rest`, returning the line and the
    /// input that follows it.
    pub(crate) fn parse(rest: &'a str) -> Result<(Self, &'a str), VcardParseError> {
        let bytes = rest.as_bytes();

        let Some(lf) = memchr::memchr(b'\n', bytes) else {
            return Err(VcardParseError::MissingCrlf(rest.to_string()));
        };

        let tail = &rest[lf + 1..];

        let (content, eol) = if lf > 0 && bytes[lf - 1] == b'\r' {
            (&rest[..lf - 1], &rest[lf - 1..lf + 1])
        } else {
            (&rest[..lf], &rest[lf..lf + 1])
        };

        let Some(colon) = memchr::memchr(b':', content.as_bytes()) else {
            return Err(VcardParseError::MissingPropertyColon(content.to_string()));
        };

        let line = Self {
            head: &content[..colon],
            value: &content[colon + 1..],
            eol,
        };

        Ok((line, tail))
    }
}
