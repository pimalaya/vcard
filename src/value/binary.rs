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

#[cfg(feature = "base64")]
use base64_codec::codec::Base64Codec;

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
    /// Decode the inline [`Base64`](Self::Base64) payload to raw bytes, with
    /// whether it was well-formed; `None` for a [`Uri`](Self::Uri) reference,
    /// which embeds no data. Requires the `base64` feature.
    ///
    /// Decoding is liberal and runs the backend of the CPU token the
    /// application installed through simd-cpu: line breaks and blanks are
    /// skipped, any other byte outside the alphabet is skipped and reported,
    /// and missing padding is accepted.
    pub fn decode_base64(&self) -> Option<(Vec<u8>, bool)> {
        let VcardBinary::Base64(data) = self else {
            return None;
        };

        let mut bytes = Vec::new();
        let clean = Base64Codec::default().decode_vec(data.as_bytes(), &mut bytes);
        Some((bytes, clean))
    }
}

#[cfg(all(test, feature = "base64"))]
mod tests {
    use alloc::{borrow::Cow, vec::Vec};

    use crate::value::binary::VcardBinary;

    #[test]
    fn decodes_inline_base64_but_not_a_uri() {
        let inline = VcardBinary::Base64(Cow::Borrowed("Zm9v"));
        assert_eq!(inline.decode_base64(), Some((Vec::from(*b"foo"), true)));

        let dirty = VcardBinary::Base64(Cow::Borrowed("Zm9v*"));
        assert_eq!(dirty.decode_base64(), Some((Vec::from(*b"foo"), false)));

        let reference = VcardBinary::Uri(Cow::Borrowed("http://example.com/p.png"));
        assert!(reference.decode_base64().is_none());
    }
}
