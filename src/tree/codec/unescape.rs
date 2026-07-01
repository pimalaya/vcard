//! # Unescape (read codec)
//!
//! Resolve the RFC 6350 3.4 value escapes when parsing. The read half of the
//! escaping codec; its exact inverse is
//! [`escape`](crate::tree::codec::escape), and the version-specific rules are
//! selected by the [`Escaper`]. The structural decoders in
//! [`decode`](crate::tree::codec::decode) run every value leaf through here.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::codec::mode::Escaper;

/// Resolve value escapes by the card's escaping mode, reading raw value bytes
/// and yielding the decoded text (lossily when the bytes are not UTF-8; the
/// caller keeps the raw bytes on the syntax leaf for fidelity).
pub(crate) fn unescape_with(bytes: &[u8], escaper: Escaper) -> Cow<'_, str> {
    lossy(unescape_bytes(bytes, escaper))
}

/// Resolve value escapes by the card's escaping mode at the byte level,
/// preserving any non-UTF-8 content verbatim.
pub(crate) fn unescape_bytes(bytes: &[u8], escaper: Escaper) -> Cow<'_, [u8]> {
    match escaper {
        Escaper::Modern => unescape_modern(bytes),
        Escaper::V2_1 => unescape_v21(bytes),
    }
}

/// Resolve value escapes with the modern (RFC 2426 / 6350) rules over text. The
/// default used wherever the escaping mode is not version-specific (parameters,
/// the version-blind lens path).
pub(crate) fn unescape(text: &str) -> Cow<'_, str> {
    lossy(unescape_modern(text.as_bytes()))
}

/// Interpret unescaped bytes as UTF-8, keeping the borrow when possible.
fn lossy(bytes: Cow<'_, [u8]>) -> Cow<'_, str> {
    match bytes {
        Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
        Cow::Owned(bytes) => Cow::Owned(String::from_utf8_lossy(&bytes).into_owned()),
    }
}

/// Resolve the RFC 2426 / 6350 3.4 value escapes `\\` `\,` `\;` `\n`, borrowing
/// when there is nothing to unescape.
fn unescape_modern(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.contains(&b'\\') {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'\\' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        match bytes.get(i + 1) {
            Some(b'n' | b'N') => out.push(b'\n'),
            Some(&other) => out.push(other),
            None => out.push(b'\\'),
        }
        i += 2;
    }

    Cow::Owned(out)
}

/// Resolve the vCard 2.1 value escape `\;` only; a backslash before anything
/// else stays literal.
fn unescape_v21(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.contains(&b'\\') {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'\\' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        match bytes.get(i + 1) {
            Some(b';') => {
                out.push(b';');
                i += 2;
            }
            Some(&other) => {
                out.push(b'\\');
                out.push(other);
                i += 2;
            }
            None => {
                out.push(b'\\');
                i += 1;
            }
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::tree::codec::unescape::unescape;

    #[test]
    fn unescapes_value_escapes_and_borrows_when_clean() {
        assert_eq!(unescape(r"a\,b\;c\nd"), "a,b;c\nd");
        assert!(matches!(unescape("plain"), Cow::Borrowed("plain")));
    }
}
