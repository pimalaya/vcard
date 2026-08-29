//! # Escape (write codec)
//!
//! Apply the RFC 6350 3.4 value escapes when serializing. The write half of the
//! escaping codec; its exact inverse is
//! [`unescape`](crate::tree::codec::unescape), and the version-specific rules
//! are selected by the [`VcardEscaper`]. The structural encoders in
//! [`encode`](crate::tree::codec::encode) run every value leaf through here.

use alloc::{borrow::Cow, vec::Vec};

use crate::tree::codec::mode::VcardEscaper;

/// Apply the value escapes by the card's escaping mode (RFC 6350 3.4 for the
/// modern rules; vCard 2.1 escapes only `;`), over raw value bytes. Borrows
/// when nothing needs escaping; non-UTF-8 content passes through verbatim.
pub(crate) fn escape_with(bytes: &[u8], escaper: VcardEscaper) -> Cow<'_, [u8]> {
    match escaper {
        VcardEscaper::Modern => escape_modern(bytes),
        VcardEscaper::V2_1 => escape_v21(bytes),
    }
}

/// Apply the RFC 2426 / 6350 3.4 value escapes `\\` `\,` `\;` `\n`.
fn escape_modern(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes
        .iter()
        .any(|b| matches!(b, b'\\' | b',' | b';' | b'\n'))
    {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len());

    for &b in bytes {
        match b {
            b'\\' => out.extend_from_slice(b"\\\\"),
            b',' => out.extend_from_slice(b"\\,"),
            b';' => out.extend_from_slice(b"\\;"),
            b'\n' => out.extend_from_slice(b"\\n"),
            other => out.push(other),
        }
    }

    Cow::Owned(out)
}

/// Apply the vCard 2.1 value escapes `\;` and `\n`.
///
/// vCard 2.1 defines only the semicolon escape, so this half is deliberately
/// not the inverse of the reader: a line break has no 2.1 spelling, and
/// emitted raw it would end the content line and cut the value in two, so it
/// is written `\n`, which a 2.1 reader (this crate's included) then reads as a
/// literal backslash. A value the wire cannot hold is the one thing worse than
/// a value it holds imprecisely.
fn escape_v21(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.iter().any(|b| matches!(b, b';' | b'\n')) {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len() + 2);

    for &b in bytes {
        match b {
            b';' => out.extend_from_slice(br"\;"),
            b'\n' => out.extend_from_slice(br"\n"),
            other => out.push(other),
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::tree::codec::{escape::escape_with, mode::VcardEscaper};

    #[test]
    fn escapes_separators_and_newlines_and_borrows_when_clean() {
        assert_eq!(
            escape_with(b"a,b;c\nd", VcardEscaper::Modern).as_ref(),
            br"a\,b\;c\nd".as_slice(),
        );
        assert!(matches!(
            escape_with(b"plain", VcardEscaper::Modern),
            Cow::Borrowed(b"plain")
        ));
        // NOTE: vCard 2.1 escapes only `;`.
        assert_eq!(
            escape_with(b"a,b;c", VcardEscaper::V2_1).as_ref(),
            br"a,b\;c".as_slice(),
        );
    }

    /// A line break is never written raw, whatever the version: it would end
    /// the content line and cut the value in two, leaving a card that does not
    /// parse back.
    #[test]
    fn a_line_break_is_escaped_in_every_version() {
        assert_eq!(
            escape_with(b"a\nb", VcardEscaper::Modern).as_ref(),
            br"a\nb".as_slice(),
        );
        // NOTE: vCard 2.1 has no line-break escape, so this half is not the
        // reader's inverse: `\n` is the only spelling that keeps the line
        // whole, and 2.1 reads it back as a literal backslash.
        assert_eq!(
            escape_with(b"a\nb", VcardEscaper::V2_1).as_ref(),
            br"a\nb".as_slice(),
        );
    }

    /// A literal backslash doubles on the way out and resolves on the way back,
    /// so a value carrying one survives a write it did not before.
    #[test]
    fn a_literal_backslash_doubles_and_resolves_back() {
        use crate::tree::codec::unescape::unescape_with;

        assert_eq!(
            escape_with(br"C:\path", VcardEscaper::Modern).as_ref(),
            br"C:\\path".as_slice(),
        );
        assert_eq!(
            unescape_with(br"C:\\path", VcardEscaper::Modern),
            r"C:\path",
        );

        // A value ending in a backslash is escaped whole, not left dangling.
        assert_eq!(
            escape_with(br"trailing\", VcardEscaper::Modern).as_ref(),
            br"trailing\\".as_slice(),
        );
        assert_eq!(
            unescape_with(br"dangling\", VcardEscaper::Modern),
            r"dangling\"
        );
    }
}
