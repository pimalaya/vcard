//! # Escape (write codec)
//!
//! Apply the RFC 6350 3.4 value escapes when serializing. The write half of the
//! escaping codec; its exact inverse is
//! [`unescape`](crate::tree::codec::unescape), and the version-specific rules
//! are selected by the [`Escaper`]. The structural encoders in
//! [`encode`](crate::tree::codec::encode) run every value leaf through here.

use alloc::{borrow::Cow, string::String};

use crate::tree::codec::mode::Escaper;

/// Apply the value escapes by the card's escaping mode (RFC 6350 3.4 for the
/// modern rules; vCard 2.1 escapes only `;`). Borrows when nothing needs
/// escaping.
pub(crate) fn escape_with(text: &str, escaper: Escaper) -> Cow<'_, str> {
    match escaper {
        Escaper::Modern => escape_modern(text),
        Escaper::V2_1 => escape_v21(text),
    }
}

/// Apply the RFC 2426 / 6350 3.4 value escapes `\\` `\,` `\;` `\n`.
fn escape_modern(text: &str) -> Cow<'_, str> {
    if !text
        .bytes()
        .any(|b| matches!(b, b'\\' | b',' | b';' | b'\n'))
    {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ',' => out.push_str("\\,"),
            ';' => out.push_str("\\;"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }

    Cow::Owned(out)
}

/// Apply the vCard 2.1 value escape: only `;` is escaped (`\;`).
fn escape_v21(text: &str) -> Cow<'_, str> {
    if !text.contains(';') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len() + 2);

    for c in text.chars() {
        if c == ';' {
            out.push('\\');
        }
        out.push(c);
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::tree::codec::{escape::escape_with, mode::Escaper};

    #[test]
    fn escapes_separators_and_newlines_and_borrows_when_clean() {
        assert_eq!(escape_with("a,b;c\nd", Escaper::Modern), r"a\,b\;c\nd");
        assert!(matches!(
            escape_with("plain", Escaper::Modern),
            Cow::Borrowed("plain")
        ));
        // vCard 2.1 escapes only `;`.
        assert_eq!(escape_with("a,b;c", Escaper::V2_1), r"a,b\;c");
    }
}
