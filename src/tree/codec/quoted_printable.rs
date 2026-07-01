//! # Quoted-printable (read codec)
//!
//! Decode the `=XX` octet encoding vCard 2.1 (and 3.0) uses for non-ASCII
//! values, keyed off the `ENCODING=QUOTED-PRINTABLE` parameter. Read-only: the
//! encoders never emit it. The value-level dispatch that decides which value
//! kinds to run through here lives in [`decode`](crate::tree::codec::decode).

use alloc::{borrow::Cow, string::String, vec::Vec};

/// Decode QUOTED-PRINTABLE `=XX` octets (soft line breaks are already joined by
/// the tokeniser). Bytes are reassembled then read as UTF-8 (lossy).
pub(crate) fn qp_decode(input: Cow<'_, str>) -> Cow<'_, str> {
    if !input.as_bytes().contains(&b'=') {
        return input;
    }

    Cow::Owned(String::from_utf8_lossy(&qp_decode_bytes(input.as_bytes())).into_owned())
}

/// Decode QUOTED-PRINTABLE `=XX` octets at the byte level, keeping the raw bytes
/// (for a value in a foreign charset). Borrows when there is no `=` to decode.
pub(crate) fn qp_decode_bytes(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.contains(&b'=') {
        return Cow::Borrowed(bytes);
    }

    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'='
            && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2]))
        {
            out.push(hi << 4 | lo);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }

    Cow::Owned(out)
}

/// The value of a hex digit, or `None`.
fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
