//! # Binary value
//!
//! The decoded binary value kind.
//!
//! Backs the binary-bearing properties (`PHOTO`, `LOGO`, `SOUND`, `KEY`) in
//! vCard 2.1 and 3.0, where the value is either an external URI reference or
//! inline base64 (the URI/binary value, RFC 6350 6.9).
//!
//! vCard 4.0 carries these as `data:` URIs instead, decoded to
//! [`VcardUri`](crate::value::uri::VcardUri). The form is told by the line's
//! `VALUE` / `ENCODING` parameters; the payload is kept verbatim (base64 is
//! not decoded to bytes).

use alloc::borrow::Cow;
#[cfg(feature = "base64")]
use alloc::vec::Vec;

/// A decoded binary value: an external URI reference, or inline base64 kept as
/// its raw text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardBinary<'a> {
    /// An external URI reference.
    Uri(Cow<'a, str>),
    /// Inline base64 data, kept verbatim (not decoded to bytes).
    Base64(Cow<'a, str>),
}

#[cfg(feature = "base64")]
impl VcardBinary<'_> {
    /// Decode the inline [`Base64`](Self::Base64) payload to raw bytes; `None`
    /// for a [`Uri`](Self::Uri) reference, which embeds no data. Requires the
    /// `base64` feature.
    pub fn decode_base64(&self) -> Option<Result<Vec<u8>, base64::DecodeError>> {
        use base64::prelude::{BASE64_STANDARD, Engine};

        match self {
            VcardBinary::Base64(data) => Some(BASE64_STANDARD.decode(data.as_bytes())),
            VcardBinary::Uri(_) => None,
        }
    }
}

#[cfg(all(test, feature = "base64"))]
mod tests {
    use alloc::borrow::Cow;

    use crate::value::binary::VcardBinary;

    #[test]
    fn decodes_inline_base64_but_not_a_uri() {
        let inline = VcardBinary::Base64(Cow::Borrowed("Zm9v"));
        assert_eq!(inline.decode_base64().unwrap().unwrap(), b"foo");

        let reference = VcardBinary::Uri(Cow::Borrowed("http://example.com/p.png"));
        assert!(reference.decode_base64().is_none());
    }
}
