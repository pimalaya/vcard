//! # Value node
//!
//! The raw value of a content line, on the syntax side.
//!
//! [`VcardValueNode`] is the syntactic peer of the decoded
//! [`VcardValue`](crate::value::VcardValue): the bytes after a line's colon,
//! understood as `;`-separated components of `,`-separated
//! [`VcardValueLeaf`](crate::tree::leaf::VcardValueLeaf) values (raw bytes, so
//! a foreign charset survives). Straight from parse the value is kept as one
//! unsplit `raw` slice and only walked on demand; an edit or a model encode
//! splits it into owned `components`, which then become the source of truth.
//! Either way the splitting is generic (it counts and preserves separators so
//! the value round-trips); what those components *mean* is the lens's business.
//! The codec that unescapes components into decoded values ([`decode_at`],
//! [`decode_scalar_at`]) and re-escapes edits back ([`set_at`]) lives on this
//! type; the escaping rules it applies come from the sibling
//! [`mode`](crate::tree::codec::mode) codec.
//!
//! [`decode_at`]: VcardValueNode::decode_at
//! [`decode_scalar_at`]: VcardValueNode::decode_scalar_at
//! [`set_at`]: VcardValueNode::set_at

use core::fmt;

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::{
    codec::{
        encode::encode_component,
        escape::escape_with,
        mode::Escaper,
        unescape::{unescape_bytes, unescape_with},
    },
    leaf::VcardValueLeaf,
};

/// A raw value: `;`-separated components, each a list of `,`-separated raw
/// value leaves.
///
/// From parse the value is held as one unsplit [`raw`](Self::raw) slice and
/// walked lazily, so a parse that never decodes and a byte-faithful reserialize
/// do no splitting at all. The first edit (or a model encode) splits it into
/// owned [`components`](Self::components), which then take over as the source of
/// truth. The `escaper` records which version's escaping rules the codec must
/// apply; it is stamped from the card version after parsing (see
/// `VcardCst::parse`).
#[derive(Clone, Debug, Default)]
pub struct VcardValueNode<'a> {
    /// The unsplit value bytes, straight from parse and authoritative until an
    /// edit or a model encode replaces them; `None` once `components` is the
    /// source of truth.
    raw: Option<Cow<'a, [u8]>>,
    /// The split components, authoritative when `raw` is `None`; while `raw` is
    /// `Some` this stays empty and reads split `raw` on demand.
    components: Vec<Vec<VcardValueLeaf<'a>>>,
    /// The escaping rules to read and write this value with.
    pub escaper: Escaper,
}

impl<'a> VcardValueNode<'a> {
    /// Wrap a raw value, unsplit and borrowed, to be walked lazily. The colon,
    /// name and eol are the line's business; this is only the bytes after the
    /// colon.
    pub fn parse(value: &'a [u8]) -> Self {
        Self {
            raw: Some(Cow::Borrowed(value)),
            components: Vec::new(),
            escaper: Escaper::default(),
        }
    }

    /// Build a value node from already-split components (the model encode path);
    /// there is no `raw`, so the components are the source of truth.
    pub(crate) fn from_components(
        components: Vec<Vec<VcardValueLeaf<'a>>>,
        escaper: Escaper,
    ) -> Self {
        Self {
            raw: None,
            components,
            escaper,
        }
    }

    /// The number of `;`-separated components (at least one, since an empty
    /// value is one empty component).
    pub fn component_count(&self) -> usize {
        match &self.raw {
            Some(raw) => {
                let mut count = 0;
                split_on(raw, b';', |_| count += 1);
                count
            }
            None => self.components.len(),
        }
    }

    /// Decode the `i`th component into a clean (unescaped) value list.
    pub fn decode_at(&self, i: usize) -> Vec<Cow<'_, str>> {
        match self.component_at(i) {
            Some(Component::Raw(bytes)) => {
                let mut values = Vec::new();
                split_on(bytes, b',', |value| {
                    values.push(unescape_with(value, self.escaper))
                });
                values
            }
            Some(Component::Split(leaves)) => leaves
                .iter()
                .map(|leaf| unescape_with(leaf.as_bytes(), self.escaper))
                .collect(),
            None => Vec::new(),
        }
    }

    /// The `i`th component's first value as raw unescaped bytes, not
    /// transcoded, for a value carrying a foreign charset.
    pub fn decode_bytes_at(&self, i: usize) -> Cow<'_, [u8]> {
        match self.component_at(i) {
            Some(Component::Raw(bytes)) => unescape_bytes(first_value(bytes), self.escaper),
            Some(Component::Split(leaves)) => leaves
                .first()
                .map(|leaf| unescape_bytes(leaf.as_bytes(), self.escaper))
                .unwrap_or(Cow::Borrowed(b"")),
            None => Cow::Borrowed(b""),
        }
    }

    /// Decode the `i`th component's first value (empty when there is none).
    pub fn decode_scalar_at(&self, i: usize) -> Cow<'_, str> {
        match self.component_at(i) {
            Some(Component::Raw(bytes)) => unescape_with(first_value(bytes), self.escaper),
            Some(Component::Split(leaves)) => leaves
                .first()
                .map(|leaf| unescape_with(leaf.as_bytes(), self.escaper))
                .unwrap_or(Cow::Borrowed("")),
            None => Cow::Borrowed(""),
        }
    }

    /// Decode the `i`th component as a single value, keeping its `,`-separated
    /// pieces joined. For values like URIs whose comma is a literal part of the
    /// value, not a list separator (so they must not be truncated).
    pub fn decode_joined_at(&self, i: usize) -> Cow<'_, str> {
        match self.component_at(i) {
            // The whole component slice already has the commas in place, so
            // unescaping it verbatim keeps them literal.
            Some(Component::Raw(bytes)) => unescape_with(bytes, self.escaper),
            Some(Component::Split(leaves)) => {
                if leaves.len() <= 1 {
                    return leaves
                        .first()
                        .map(|leaf| unescape_with(leaf.as_bytes(), self.escaper))
                        .unwrap_or(Cow::Borrowed(""));
                }

                let mut raw = Vec::new();
                for (j, leaf) in leaves.iter().enumerate() {
                    if j > 0 {
                        raw.push(b',');
                    }
                    raw.extend_from_slice(leaf.as_bytes());
                }

                Cow::Owned(unescape_with(&raw, self.escaper).into_owned())
            }
            None => Cow::Borrowed(""),
        }
    }

    /// The raw (still-escaped) bytes of the first component's first value, for
    /// the simple single-value lines (the envelope values and diagnostics).
    pub(crate) fn first_value_bytes(&self) -> &[u8] {
        match self.component_at(0) {
            Some(Component::Raw(bytes)) => first_value(bytes),
            Some(Component::Split(leaves)) => {
                leaves.first().map(|leaf| leaf.as_bytes()).unwrap_or(b"")
            }
            None => b"",
        }
    }

    /// Set the `i`th component, escaping each value. Pads with empty components
    /// when needed; every other component is left untouched, so a parsed card
    /// keeps its bytes.
    pub fn set_at<S: AsRef<str>>(&mut self, i: usize, values: &[S]) {
        self.materialize();

        while self.components.len() <= i {
            self.components.push(Vec::new());
        }

        self.components[i] = encode_component(values, self.escaper);
    }

    /// Set the `i`th component from raw value bytes (the foreign-charset escape
    /// hatch), escaping structural separators but writing the bytes verbatim.
    pub fn set_bytes_at<B: AsRef<[u8]>>(&mut self, i: usize, values: &[B]) {
        self.materialize();

        while self.components.len() <= i {
            self.components.push(Vec::new());
        }

        self.components[i] = values
            .iter()
            .map(|v| VcardValueLeaf::from(escape_with(v.as_ref(), self.escaper).into_owned()))
            .collect();
    }

    /// Serialize the raw value bytes (name, colon and eol are the line's job)
    /// into `out`, exactly as parsed. An untouched value emits its `raw` slice
    /// with no reassembly.
    pub(crate) fn write_bytes(&self, out: &mut Vec<u8>) {
        if let Some(raw) = &self.raw {
            out.extend_from_slice(raw);
            return;
        }

        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                out.push(b';');
            }

            for (j, leaf) in component.iter().enumerate() {
                if j > 0 {
                    out.push(b',');
                }

                out.extend_from_slice(leaf.as_bytes());
            }
        }
    }

    /// Convert into an owned value node (`'static`), keeping it lazy: an
    /// unsplit value stays unsplit, only its bytes become owned.
    pub(crate) fn into_static(self) -> VcardValueNode<'static> {
        match self.raw {
            Some(raw) => VcardValueNode {
                raw: Some(Cow::Owned(raw.into_owned())),
                components: Vec::new(),
                escaper: self.escaper,
            },
            None => VcardValueNode {
                raw: None,
                components: self
                    .components
                    .into_iter()
                    .map(|component| {
                        component
                            .into_iter()
                            .map(VcardValueLeaf::into_static)
                            .collect()
                    })
                    .collect(),
                escaper: self.escaper,
            },
        }
    }

    /// Locate the `i`th component, either as a raw slice of the unsplit value
    /// or as the already-split leaves.
    fn component_at(&self, i: usize) -> Option<Component<'_, 'a>> {
        match &self.raw {
            Some(raw) => {
                let mut found = None;
                let mut index = 0;
                split_on(raw, b';', |component| {
                    if index == i {
                        found = Some(component);
                    }
                    index += 1;
                });
                found.map(Component::Raw)
            }
            None => self
                .components
                .get(i)
                .map(|leaves| Component::Split(leaves)),
        }
    }

    /// Split the unsplit `raw` value into owned components so it can be edited
    /// in place; a no-op once already split. Only edits pay this cost.
    fn materialize(&mut self) {
        let Some(raw) = self.raw.take() else {
            return;
        };

        self.components = match raw {
            Cow::Borrowed(bytes) => split_all(bytes),
            Cow::Owned(bytes) => split_all_owned(&bytes),
        };
    }
}

impl fmt::Display for VcardValueNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(raw) = &self.raw {
            return f.write_str(&String::from_utf8_lossy(raw));
        }

        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                f.write_str(";")?;
            }

            for (j, leaf) in component.iter().enumerate() {
                if j > 0 {
                    f.write_str(",")?;
                }

                f.write_str(&String::from_utf8_lossy(leaf.as_bytes()))?;
            }
        }

        Ok(())
    }
}

/// One located component: a slice of the still-unsplit value, or its leaves
/// once the value has been split for editing.
enum Component<'s, 'a> {
    /// The component's bytes, still `,`-joined, borrowed from the unsplit value.
    Raw(&'s [u8]),
    /// The component's already-split leaves.
    Split(&'s [VcardValueLeaf<'a>]),
}

/// The first `,`-separated value of a component (escape-aware): the bytes up to
/// the first unescaped comma, or the whole component when it has none.
fn first_value(component: &[u8]) -> &[u8] {
    let mut first = None;
    split_on(component, b',', |value| {
        first.get_or_insert(value);
    });
    first.unwrap_or(component)
}

/// Split a value into its `;`-separated components, each a list of its
/// `,`-separated leaves, borrowing from `bytes`.
fn split_all(bytes: &[u8]) -> Vec<Vec<VcardValueLeaf<'_>>> {
    let mut components = Vec::new();
    split_on(bytes, b';', |component| {
        let mut values = Vec::new();
        split_on(component, b',', |value| {
            values.push(VcardValueLeaf::from(value));
        });
        components.push(values);
    });
    components
}

/// Split like [`split_all`] but copying each leaf, for the owned bytes an
/// edit-after-`into_static` value carries (which no slice can borrow from).
fn split_all_owned(bytes: &[u8]) -> Vec<Vec<VcardValueLeaf<'static>>> {
    let mut components = Vec::new();
    split_on(bytes, b';', |component| {
        let mut values = Vec::new();
        split_on(component, b',', |value| {
            values.push(VcardValueLeaf::from(value.to_vec()));
        });
        components.push(values);
    });
    components
}

/// Call `piece` for each span between unescaped `sep` bytes, always at least
/// once. A backslash escapes the next byte (so `\;` / `\,` do not split), and
/// `memchr` skips straight to the next `sep` or backslash instead of scanning
/// byte by byte, so a large separator-free value (e.g. base64) is skipped in
/// one pass.
fn split_on<'b>(bytes: &'b [u8], sep: u8, mut piece: impl FnMut(&'b [u8])) {
    let mut start = 0;
    let mut i = 0;

    while let Some(offset) = memchr::memchr2(b'\\', sep, &bytes[i..]) {
        let pos = i + offset;
        if bytes[pos] == b'\\' {
            i = (pos + 2).min(bytes.len());
        } else {
            piece(&bytes[start..pos]);
            start = pos + 1;
            i = pos + 1;
        }
    }

    piece(&bytes[start..]);
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::tree::value::VcardValueNode;

    #[test]
    fn splits_components_and_values_then_round_trips() {
        let node = VcardValueNode::parse(b"a;b,c;");
        assert_eq!(node.component_count(), 3);
        assert_eq!(
            node.decode_at(1),
            vec![Cow::Borrowed("b"), Cow::Borrowed("c")]
        );
        assert_eq!(node.to_string(), "a;b,c;");
    }

    #[test]
    fn keeps_escaped_separators_inside_one_value() {
        let node = VcardValueNode::parse(br"a\,b\;c;d");
        assert_eq!(node.component_count(), 2);
        assert_eq!(node.decode_at(0).len(), 1);
        assert_eq!(node.to_string(), r"a\,b\;c;d");
    }

    #[test]
    fn an_edit_splits_and_preserves_untouched_components() {
        let mut node = VcardValueNode::parse(b"a;b;c");
        node.set_at(1, &["X"]);
        assert_eq!(node.to_string(), "a;X;c");
    }
}
