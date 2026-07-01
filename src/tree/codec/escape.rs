//! # Escape (write codec)
//!
//! Apply the RFC 6350 3.4 value escapes when serializing. The write half of the
//! escaping codec; its exact inverse is
//! [`unescape`](crate::tree::codec::unescape), and the version-specific rules
//! are selected by the [`Escaper`]. The structural encoders in
//! [`encode`](crate::tree::codec::encode) run every value leaf through here.

use alloc::{borrow::Cow, vec::Vec};

use crate::tree::codec::mode::Escaper;

/// Apply the value escapes by the card's escaping mode (RFC 6350 3.4 for the
/// modern rules; vCard 2.1 escapes only `;`), over raw value bytes. Borrows when
/// nothing needs escaping; non-UTF-8 content passes through verbatim.
pub(crate) fn escape_with(bytes: &[u8], escaper: Escaper) -> Cow<'_, [u8]> {
    match escaper {
        Escaper::Modern => escape_modern(bytes),
        Escaper::V2_1 => escape_v21(bytes),
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

/// Apply the vCard 2.1 value escape: only `;` is escaped (`\;`).
fn escape_v21(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.contains(&b';') {
        return Cow::Borrowed(bytes);
    }

    let mut out = Vec::with_capacity(bytes.len() + 2);

    for &b in bytes {
        if b == b';' {
            out.push(b'\\');
        }
        out.push(b);
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::tree::codec::{escape::escape_with, mode::Escaper};

    #[test]
    fn escapes_separators_and_newlines_and_borrows_when_clean() {
        assert_eq!(
            escape_with(b"a,b;c\nd", Escaper::Modern).as_ref(),
            br"a\,b\;c\nd".as_slice(),
        );
        assert!(matches!(
            escape_with(b"plain", Escaper::Modern),
            Cow::Borrowed(b"plain")
        ));
        // vCard 2.1 escapes only `;`.
        assert_eq!(
            escape_with(b"a,b;c", Escaper::V2_1).as_ref(),
            br"a,b\;c".as_slice(),
        );
    }
}
