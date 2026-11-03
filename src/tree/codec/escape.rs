//! # Escape (write codec)
//!
//! Apply the RFC 6350 3.4 value escapes when serializing. The write half of the
//! escaping codec; its exact inverse is
//! [`unescape`](crate::tree::codec::unescape), and the version-specific rules
//! are selected by the [`VcardEscaper`].
//!
//! The structural encoders in [`encode`](crate::tree::codec::encode) run every
//! value leaf through here.
//!
//! A parameter value is a different alphabet and has its own writer,
//! `escape_param`, applying the RFC 6868 caret encoding rather than any
//! backslash one.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::codec::mode::VcardEscaper;

/// Apply the value escapes by the card's escaping mode (RFC 6350 3.4 for the
/// modern rules; vCard 2.1 escapes `;` and a line break), over raw value
/// bytes. Borrows when nothing needs escaping; non-UTF-8 content passes
/// through verbatim.
pub(crate) fn escape_with(bytes: &[u8], escaper: VcardEscaper) -> Cow<'_, [u8]> {
    let Some(mut at) = escaped_at(bytes, escaper) else {
        return Cow::Borrowed(bytes);
    };

    let mut out = Vec::with_capacity(bytes.len() + 2);
    let mut rest = bytes;

    loop {
        out.extend_from_slice(&rest[..at]);
        out.extend_from_slice(escape_of(rest[at]).as_bytes());
        rest = &rest[at + 1..];

        match escaped_at(rest, escaper) {
            Some(next) => at = next,
            None => break,
        }
    }

    out.extend_from_slice(rest);
    Cow::Owned(out)
}

/// Append a value to `text`, escaped as [`escape_with`] escapes its bytes.
///
/// The writer of a decoded card's text: a value is a string there, and every
/// escape is ASCII, so the pieces in between stay strings.
pub(crate) fn escape_into(value: &str, escaper: VcardEscaper, text: &mut String) {
    let mut rest = value;

    while let Some(at) = escaped_at(rest.as_bytes(), escaper) {
        let (run, tail) = rest.split_at(at);
        text.push_str(run);
        text.push_str(escape_of(tail.as_bytes()[0]));
        rest = &tail[1..];
    }

    text.push_str(rest);
}

/// The index of the first byte a value escape rewrites, by escaping mode:
/// `\`, `,`, `;` and a line break from 3.0 on, `;` and a line break in 2.1.
///
/// 2.1 writes `\n` for a line break though it reads it back literally: a raw
/// one would end the content line and cut the value in two.
fn escaped_at(bytes: &[u8], escaper: VcardEscaper) -> Option<usize> {
    match escaper {
        VcardEscaper::V3_0 | VcardEscaper::V4_0 => position(bytes, |b| {
            (b == b'\\') | (b == b',') | (b == b';') | (b == b'\n')
        }),
        VcardEscaper::V2_1 => memchr::memchr2(b';', b'\n', bytes),
    }
}

/// The escape written for a byte [`escaped_at`] finds.
fn escape_of(byte: u8) -> &'static str {
    match byte {
        b'\\' => r"\\",
        b',' => r"\,",
        b';' => r"\;",
        _ => r"\n",
    }
}

/// The index of the first byte `hit` matches.
///
/// Sixteen bytes at a time, each block's hits gathered into a bit mask with
/// no early exit, which compiles to vector compares and a mask move, so a
/// long base64 value passes at several bytes a cycle. `hit` must join its
/// tests with `|`: a `matches!` compiles to a branch per byte.
pub(crate) fn position(bytes: &[u8], hit: impl Fn(u8) -> bool) -> Option<usize> {
    let (blocks, tail) = bytes.as_chunks::<16>();

    for (i, block) in blocks.iter().enumerate() {
        let mut mask = 0u16;

        for (j, &b) in block.iter().enumerate() {
            mask |= u16::from(hit(b)) << j;
        }

        if mask != 0 {
            return Some(i * 16 + mask.trailing_zeros() as usize);
        }
    }

    tail.iter()
        .position(|&b| hit(b))
        .map(|j| blocks.len() * 16 + j)
}

/// Apply the RFC 6868 3.1 parameter value encoding: `^n`, `^^` and `^'`, then
/// wrap the result in the RFC 6350 3.3 delimiters when it needs them.
///
/// Inverse of [`unescape_param`](crate::tree::codec::unescape::unescape_param).
pub(crate) fn escape_param(value: &str, escaper: VcardEscaper) -> Cow<'_, str> {
    let value = if escaper.has_param_encoding() {
        escape_carets(value)
    } else {
        Cow::Borrowed(value)
    };

    if !escaper.has_param_quoting() || !value.bytes().any(|b| matches!(b, b',' | b';' | b':')) {
        return value;
    }

    // NOTE: a double quote never reaches here in 4.0, the caret encoding
    // having spelled it `^'`. In 3.0 it cannot be encoded at all, so a value
    // holding one alongside a delimiter has no conformant spelling and is
    // written quoted rather than left to split on the delimiter.
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    out.push_str(&value);
    out.push('"');
    Cow::Owned(out)
}

/// Append a parameter value to `text`, encoded as [`escape_param`] encodes it.
///
/// The writer of a decoded card's text, which writes the value in place: an
/// encoded value returned through memory is copied by loads that wait on its
/// stores. The caret encoding adds no delimiter, so whether the value needs
/// quoting is read off the value as given.
pub(crate) fn escape_param_into(value: &str, escaper: VcardEscaper, text: &mut String) {
    let quoted = escaper.has_param_quoting()
        && value
            .bytes()
            .any(|b| (b == b',') | (b == b';') | (b == b':'));

    if quoted {
        text.push('"');
    }

    if escaper.has_param_encoding()
        && value
            .bytes()
            .any(|b| (b == b'\n') | (b == b'^') | (b == b'"'))
    {
        text.push_str(&escape_carets(value));
    } else {
        text.push_str(value);
    }

    if quoted {
        text.push('"');
    }
}

/// Apply the RFC 6868 caret encoding over every character of `value`.
fn escape_carets(value: &str) -> Cow<'_, str> {
    if !value.bytes().any(|b| matches!(b, b'\n' | b'^' | b'"')) {
        return Cow::Borrowed(value);
    }

    let mut out = String::with_capacity(value.len());

    for c in value.chars() {
        match c {
            '\n' => out.push_str("^n"),
            '^' => out.push_str("^^"),
            '"' => out.push_str("^'"),
            other => out.push(other),
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::tree::codec::{
        escape::{escape_param, escape_with},
        mode::VcardEscaper,
    };

    #[test]
    fn escapes_separators_and_newlines_and_borrows_when_clean() {
        assert_eq!(
            escape_with(b"a,b;c\nd", VcardEscaper::V4_0).as_ref(),
            br"a\,b\;c\nd".as_slice(),
        );
        assert!(matches!(
            escape_with(b"plain", VcardEscaper::V4_0),
            Cow::Borrowed(b"plain")
        ));
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
            escape_with(b"a\nb", VcardEscaper::V4_0).as_ref(),
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
            escape_with(br"C:\path", VcardEscaper::V4_0).as_ref(),
            br"C:\\path".as_slice(),
        );
        assert_eq!(unescape_with(br"C:\\path", VcardEscaper::V4_0), r"C:\path",);

        assert_eq!(
            escape_with(br"trailing\", VcardEscaper::V4_0).as_ref(),
            br"trailing\\".as_slice(),
        );
        assert_eq!(
            unescape_with(br"dangling\", VcardEscaper::V4_0),
            r"dangling\"
        );
    }

    #[test]
    fn encodes_the_rfc_6868_parameter_sequences_and_borrows_when_clean() {
        assert_eq!(escape_param("a\nb^c\"d", VcardEscaper::V4_0), "a^nb^^c^'d",);
        assert!(matches!(
            escape_param("plain", VcardEscaper::V4_0),
            Cow::Borrowed("plain")
        ));
        // NOTE: RFC 6868 section 3.2 forbids backslash escaping, so a path
        // keeps its backslash; its colon is what the quotes are for.
        assert_eq!(escape_param(r"C:\temp", VcardEscaper::V4_0), r#""C:\temp""#);
    }

    /// RFC 6350 section 3.3 keeps `,`, `;` and `:` out of a bare SAFE-CHAR
    /// run, so a value carrying one is wrapped and a value carrying none is
    /// not: the quotes are the grammar's, not the value's.
    #[test]
    fn quotes_a_parameter_value_only_where_a_delimiter_needs_it() {
        assert_eq!(
            escape_param("geo:37.386,-122.083", VcardEscaper::V4_0),
            "\"geo:37.386,-122.083\"",
        );
        assert_eq!(escape_param("05:45", VcardEscaper::V3_0), "\"05:45\"");
        assert!(matches!(
            escape_param("work", VcardEscaper::V4_0),
            Cow::Borrowed("work")
        ));
    }

    /// A double quote is content, so it goes out RFC 6868 encoded rather than
    /// as a delimiter, and the pair the comma calls for is added around it.
    #[test]
    fn encodes_a_double_quote_rather_than_reading_it_as_a_delimiter() {
        assert_eq!(
            escape_param("say \"hi\", then go", VcardEscaper::V4_0),
            "\"say ^'hi^', then go\"",
        );
    }

    /// vCard 2.1 has no quoted-string, so nothing is wrapped: a delimiter goes
    /// out bare, as every 2.1 writer puts it.
    #[test]
    fn never_quotes_a_2_1_parameter_value() {
        assert!(matches!(
            escape_param("a,b", VcardEscaper::V2_1),
            Cow::Borrowed("a,b")
        ));
    }

    /// RFC 6868 updates RFC 6350 alone, so a 2.1 or 3.0 caret goes out as
    /// itself, and neither reader would resolve `^^` anyway.
    #[test]
    fn writes_a_pre_4_0_parameter_unencoded() {
        assert!(matches!(
            escape_param("a^b", VcardEscaper::V3_0),
            Cow::Borrowed("a^b")
        ));
        assert!(matches!(
            escape_param("a^b", VcardEscaper::V2_1),
            Cow::Borrowed("a^b")
        ));
    }
}
