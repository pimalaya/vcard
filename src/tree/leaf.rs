//! # Leaf
//!
//! The atom of the syntax tree: a single raw piece of a card.
//!
//! Two leaf kinds split on a spec boundary. [`VcardLeaf`] wraps still-escaped
//! *text* (a name, a parameter value, a line ending), US-ASCII in every version
//! and so always valid UTF-8.
//!
//! [`VcardValueLeaf`] wraps a still-escaped *value* component as raw bytes,
//! because a property value may carry a foreign charset (a vCard 2.1
//! `CHARSET`) that is not UTF-8.
//!
//! Both are a [`Cow`], so a parsed leaf borrows the source (the basis of
//! byte-faithful round-trips) and only becomes owned when a build or an edit
//! replaces it.

use alloc::{borrow::Cow, string::String, vec::Vec};

/// A single raw text piece of a card (a name, a parameter value, a line
/// ending): borrowed when parsed, owned when built or edited. Always valid
/// UTF-8, since the parser rejects a non-UTF-8 name or parameter.
#[derive(Clone, Debug)]
pub struct VcardLeaf<'a>(pub Cow<'a, str>);

impl<'a> VcardLeaf<'a> {
    /// The raw (still-escaped) text of the leaf.
    pub fn get(&self) -> &str {
        &self.0
    }

    /// Replace the leaf's raw text.
    pub fn set(&mut self, text: impl Into<Cow<'a, str>>) {
        self.0 = text.into();
    }

    /// Convert into an owned leaf (`'static`), cloning the text if borrowed.
    pub(crate) fn into_static(self) -> VcardLeaf<'static> {
        VcardLeaf(Cow::Owned(self.0.into_owned()))
    }
}

impl<'a> From<&'a str> for VcardLeaf<'a> {
    fn from(text: &'a str) -> Self {
        Self(Cow::Borrowed(text))
    }
}

impl From<String> for VcardLeaf<'_> {
    fn from(text: String) -> Self {
        Self(Cow::Owned(text))
    }
}

/// A single raw *value* component of a card, held as raw bytes so a foreign
/// charset survives byte for byte. Borrowed when parsed, owned when built or
/// edited. The codec resolves these bytes to the decoded model's UTF-8 text
/// (lossily, when they are not UTF-8); the raw bytes stay reachable here.
#[derive(Clone, Debug)]
pub struct VcardValueLeaf<'a>(pub Cow<'a, [u8]>);

impl<'a> VcardValueLeaf<'a> {
    /// The raw (still-escaped) bytes of the leaf.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The raw bytes as UTF-8 text, lossily (invalid sequences become the
    /// replacement character). For a diagnostic or a best-effort read; the
    /// exact bytes are [`as_bytes`](Self::as_bytes).
    pub fn to_str_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.0)
    }

    /// Replace the leaf's raw bytes.
    pub fn set(&mut self, bytes: impl Into<Cow<'a, [u8]>>) {
        self.0 = bytes.into();
    }

    /// Convert into an owned leaf (`'static`), cloning the bytes if borrowed.
    pub(crate) fn into_static(self) -> VcardValueLeaf<'static> {
        VcardValueLeaf(Cow::Owned(self.0.into_owned()))
    }
}

impl<'a> From<&'a [u8]> for VcardValueLeaf<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(Cow::Borrowed(bytes))
    }
}

impl From<Vec<u8>> for VcardValueLeaf<'_> {
    fn from(bytes: Vec<u8>) -> Self {
        Self(Cow::Owned(bytes))
    }
}

impl<'a> From<Cow<'a, str>> for VcardValueLeaf<'a> {
    fn from(text: Cow<'a, str>) -> Self {
        Self(match text {
            Cow::Borrowed(text) => Cow::Borrowed(text.as_bytes()),
            Cow::Owned(text) => Cow::Owned(text.into_bytes()),
        })
    }
}

/// Append `bytes` to `out`, a piece of sixteen bytes or fewer by fixed-size
/// moves rather than a call.
///
/// Most pieces of a line are a few bytes (a name, a parameter, a line ending),
/// for which a call to `memcpy` costs more than the copy. The piece is written
/// over sixteen zeroes by two moves that overlap when it is shorter than
/// their sum, and the zeroes past it are cut off.
#[inline(always)]
pub(crate) fn put(out: &mut Vec<u8>, bytes: &[u8]) {
    let len = bytes.len();

    if len > 16 {
        return out.extend_from_slice(bytes);
    }

    let at = out.len();
    out.extend_from_slice(&[0; 16]);
    let to = &mut out[at..at + 16];

    if len >= 8 {
        to[..8].copy_from_slice(&bytes[..8]);
        to[len - 8..len].copy_from_slice(&bytes[len - 8..]);
    } else if len >= 4 {
        to[..4].copy_from_slice(&bytes[..4]);
        to[len - 4..len].copy_from_slice(&bytes[len - 4..]);
    } else if len > 0 {
        to[0] = bytes[0];
        to[len / 2] = bytes[len / 2];
        to[len - 1] = bytes[len - 1];
    }

    out.truncate(at + len);
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::tree::leaf::{VcardLeaf, VcardValueLeaf, put};

    /// Every length a fixed-size move covers, each piece after one that left
    /// bytes behind, comes out exactly.
    #[test]
    fn puts_every_short_length_exactly() {
        let source: Vec<u8> = (b'a'..=b'z').collect();
        let mut out = b"x".to_vec();
        let mut expected = b"x".to_vec();

        for len in 0..=source.len() {
            put(&mut out, &source[..len]);
            expected.extend_from_slice(&source[..len]);
        }

        assert_eq!(out, expected);
    }

    #[test]
    fn replaces_leaf_contents() {
        let mut text = VcardLeaf::from("a");
        text.set("b");
        assert_eq!(text.get(), "b");

        let mut bytes = VcardValueLeaf::from(b"a".as_slice());
        bytes.set(Vec::from(b"c".as_slice()));
        assert_eq!(bytes.as_bytes(), b"c");
        assert_eq!(bytes.to_str_lossy(), "c");
    }
}
