use alloc::{borrow::Cow, string::String};

/// A single piece of a card: a borrowed slice of the source when parsed, an
/// owned string when built or edited. Walking every leaf in source order, with
/// the invariant separators a node emits between them, reproduces the card.
#[derive(Clone, Debug)]
pub struct VcardLeaf<'a>(pub Cow<'a, str>);

impl<'a> VcardLeaf<'a> {
    /// The current text of the leaf.
    pub fn text(&self) -> &str {
        &self.0
    }

    /// Replace the leaf's text on the next rebuild.
    pub fn replace(&mut self, text: impl Into<Cow<'a, str>>) {
        self.0 = text.into();
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
