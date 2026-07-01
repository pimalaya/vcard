//! # Unescape (read codec)
//!
//! Resolve the RFC 6350 3.4 value escapes when parsing. The read half of the
//! escaping codec; its exact inverse is
//! [`escape`](crate::tree::codec::escape), and the version-specific rules are
//! selected by the [`Escaper`]. The structural decoders in
//! [`decode`](crate::tree::codec::decode) run every value leaf through here.

use alloc::{borrow::Cow, string::String};

use crate::tree::codec::mode::Escaper;

/// Resolve value escapes by the card's escaping mode.
pub(crate) fn unescape_with(text: &str, escaper: Escaper) -> Cow<'_, str> {
    match escaper {
        Escaper::Modern => unescape_modern(text),
        Escaper::V2_1 => unescape_v21(text),
    }
}

/// Resolve value escapes with the modern (RFC 2426 / 6350) rules. The default
/// used wherever the escaping mode is not version-specific (parameters, the
/// version-blind lens path).
pub(crate) fn unescape(text: &str) -> Cow<'_, str> {
    unescape_modern(text)
}

/// Resolve the RFC 2426 / 6350 3.4 value escapes `\\` `\,` `\;` `\n`, borrowing
/// when there is nothing to unescape.
fn unescape_modern(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }

    Cow::Owned(out)
}

/// Resolve the vCard 2.1 value escape `\;` only; a backslash before anything
/// else stays literal.
fn unescape_v21(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some(';') => out.push(';'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
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
