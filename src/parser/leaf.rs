use core::ops::Range;

use alloc::string::String;

/// A piece of the source, plus an optional edit that overrides it on rebuild.
#[derive(Clone, Debug)]
pub struct VcardLeaf {
    /// The byte range of the piece in the source input.
    pub range: Range<usize>,
    /// An edited value that replaces the source bytes on rebuild, if set.
    pub r#override: Option<String>,
}

impl VcardLeaf {
    pub fn new(range: Range<usize>) -> Self {
        Self {
            range,
            r#override: None,
        }
    }

    /// The current text: the override if set, otherwise the source bytes.
    pub fn text<'a>(&'a self, input: &'a str) -> &'a str {
        match &self.r#override {
            Some(edit) => edit,
            None => &input[self.range.clone()],
        }
    }

    /// Override this leaf's bytes with `text` on the next rebuild.
    pub fn replace(&mut self, text: impl Into<String>) {
        self.r#override = Some(text.into());
    }

    /// Drop any override, restoring the source bytes on rebuild.
    pub fn clear(&mut self) {
        self.r#override = None;
    }

    /// The override paired with the range it replaces, if one is set.
    pub fn edit(&self) -> Option<(Range<usize>, &str)> {
        let text = self.r#override.as_deref()?;
        Some((self.range.clone(), text))
    }
}
