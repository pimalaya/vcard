//! # Leaf
//!
//! The atom of the syntax tree: a single raw string piece.
//!
//! [`VcardLeaf`] wraps one still-escaped slice of a card: a name, a parameter
//! value, a single component value, a line ending. It is a [`Cow`], so a parsed
//! leaf borrows the source (the basis of byte-faithful round-trips) and only
//! becomes owned when a build or an edit replaces it. Every other syntax node
//! ([`line`](crate::v3_0::tree::line), [`param`](crate::v3_0::tree::param),
//! [`value`](crate::v3_0::tree::value)) is ultimately a tree of these.

use alloc::{borrow::Cow, string::String};

/// A single raw piece of a card: borrowed when parsed, owned when built or
/// edited.
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
