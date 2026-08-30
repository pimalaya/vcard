//! # Unescape (read codec)
//!
//! Resolve the RFC 6350 3.4 value escapes when parsing. The read half of the
//! escaping codec; its exact inverse is
//! [`escape`](crate::tree::codec::escape), and the version-specific rules are
//! selected by the [`VcardEscaper`].
//!
//! The structural decoders in [`decode`](crate::tree::codec::decode) run every
//! value leaf through here.
//!
//! A parameter value is a different alphabet and has its own reader,
//! `unescape_param`. RFC 6350 section 3.3 gives a parameter no backslash
//! escapes at all, which is why RFC 6868 gives it the caret ones instead.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::codec::mode::VcardEscaper;

/// Resolve value escapes by the card's escaping mode, reading raw value bytes
/// and yielding the decoded text (lossily when the bytes are not UTF-8; the
/// caller keeps the raw bytes on the syntax leaf for fidelity).
pub(crate) fn unescape_with(bytes: &[u8], escaper: VcardEscaper) -> Cow<'_, str> {
    lossy(unescape_bytes(bytes, escaper))
}

/// Resolve value escapes by the card's escaping mode at the byte level,
/// preserving any non-UTF-8 content verbatim.
pub(crate) fn unescape_bytes(bytes: &[u8], escaper: VcardEscaper) -> Cow<'_, [u8]> {
    match escaper {
        VcardEscaper::V3_0 | VcardEscaper::V4_0 => unescape_modern(bytes),
        VcardEscaper::V2_1 => unescape_v21(bytes),
    }
}

/// Resolve the RFC 6868 3.1 parameter value encoding: `^n`, `^^` and `^'`.
///
/// A caret before anything else, and a trailing one, stay literal, 3.1
/// forbidding an error either way. No backslash is touched: RFC 6350 3.3 gives
/// a parameter no escapes and RFC 6868 3.2 forbids adding the backslash ones.
pub(crate) fn unescape_param(text: &str, escaper: VcardEscaper) -> Cow<'_, str> {
    if !escaper.has_param_encoding() || !text.contains('^') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '^' {
            out.push(c);
            continue;
        }

        match chars.peek() {
            Some('n') => out.push('\n'),
            Some('^') => out.push('^'),
            Some('\'') => out.push('"'),
            // NOTE: any other caret sequence is left as it stands, so the
            // caret goes out alone and the character after it is read again.
            _ => {
                out.push('^');
                continue;
            }
        }

        chars.next();
    }

    Cow::Owned(out)
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

    use crate::tree::codec::{
        mode::VcardEscaper,
        unescape::{unescape_param, unescape_with},
    };

    #[test]
    fn unescapes_value_escapes_and_borrows_when_clean() {
        assert_eq!(
            unescape_with(br"a\,b\;c\nd", VcardEscaper::V4_0),
            "a,b;c\nd",
        );
        assert!(matches!(
            unescape_with(b"plain", VcardEscaper::V4_0),
            Cow::Borrowed("plain")
        ));
    }

    #[test]
    fn unescapes_the_rfc_6868_parameter_sequences() {
        assert_eq!(
            unescape_param("a^nb^^c^'d", VcardEscaper::V4_0),
            "a\nb^c\"d",
        );
        assert!(matches!(
            unescape_param("plain", VcardEscaper::V4_0),
            Cow::Borrowed("plain")
        ));
    }

    /// RFC 6868 section 3.1 forbids reading `^x` as an error, and section 3.2
    /// forbids backslash escaping, so both stay as they are.
    #[test]
    fn keeps_an_unknown_caret_sequence_and_a_backslash() {
        assert_eq!(unescape_param("a^xb^Nc^", VcardEscaper::V4_0), "a^xb^Nc^");
        assert_eq!(
            unescape_param(r"C:\temp\note", VcardEscaper::V4_0),
            r"C:\temp\note",
        );
    }

    /// RFC 6868 updates RFC 6350 alone, so a 2.1 or 3.0 caret is a literal
    /// caret and resolving it would corrupt the value.
    #[test]
    fn leaves_a_pre_4_0_parameter_caret_alone() {
        assert!(matches!(
            unescape_param("a^nb", VcardEscaper::V3_0),
            Cow::Borrowed("a^nb")
        ));
        assert!(matches!(
            unescape_param("a^nb", VcardEscaper::V2_1),
            Cow::Borrowed("a^nb")
        ));
    }

    /// vCard 2.1 resolves `\;` only: `\n` keeps its literal backslash, and a
    /// trailing backslash stays.
    #[test]
    fn unescapes_only_the_semicolon_in_v2_1() {
        assert_eq!(unescape_with(br"a\;b\nc\", VcardEscaper::V2_1), "a;b\\nc\\");
    }
}
