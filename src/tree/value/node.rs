//! # Value node
//!
//! The raw value of a content line, on the syntax side.
//!
//! [`VcardValueNode`] is the syntactic peer of the decoded
//! [`VcardValue`](crate::value::VcardValue): the bytes after a line's colon,
//! read as `;`-separated components of `,`-separated
//! [`crate::tree::leaf::VcardValueLeaf`]s.
//!
//! A leaf is raw bytes, so a foreign charset survives. Straight from parse the
//! value stays one unsplit slice walked on demand; an edit or a model encode
//! splits it into owned components, which then become the source of truth.
//!
//! The splitting is generic, counting and preserving separators so the value
//! round-trips; what the components *mean* is the lens's business.
//!
//! The codec that unescapes components into decoded values and re-escapes edits
//! back lives on this type, applying the rules of the sibling
//! [`mode`](crate::tree::codec::mode) codec.

use core::{fmt, iter, ops::Range, slice, str};

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::{
    codec::{
        encode::{encode_bytes_component, encode_component, encode_leaf},
        mode::VcardEscaper,
        unescape::{text, unescape_bytes, unescape_text, unescape_with},
    },
    leaf::VcardValueLeaf,
};

/// A raw value: `;`-separated components, each a list of `,`-separated leaves.
///
/// From parse it is one unsplit slice walked lazily; the first edit splits it
/// into owned components, and a write touches only the leaf it names, so
/// everything else keeps its parsed bytes. `escaper` records the version rules.
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
    pub escaper: VcardEscaper,
}

impl<'a> VcardValueNode<'a> {
    /// Wrap a raw value, unsplit and borrowed, to be walked lazily. The colon,
    /// name and eol are the line's business; this is only the bytes after the
    /// colon.
    pub fn parse(value: &'a [u8]) -> Self {
        Self::parse_cow(Cow::Borrowed(value))
    }

    /// Wrap a raw value, unsplit, borrowed or owned (the bytes of a folded
    /// line, joined once).
    pub(crate) fn parse_cow(value: Cow<'a, [u8]>) -> Self {
        Self {
            raw: Some(value),
            components: Vec::new(),
            escaper: VcardEscaper::default(),
        }
    }

    /// Build a value node from already-split components (the model encode
    /// path); there is no `raw`, so the components are the source of truth.
    pub(crate) const fn from_components(
        components: Vec<Vec<VcardValueLeaf<'a>>>,
        escaper: VcardEscaper,
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
            Some(raw) => pieces(raw, b';').count(),
            None => self.components.len(),
        }
    }

    /// Decode the whole value as one string, its `;` and `,` kept literal.
    ///
    /// The read to reach for. A value the specification gives no structure of
    /// its own separates nothing with its semicolon (RFC 6350 4.2), so naming
    /// a component truncates `data:text/plain;base64,AAA` at the media type.
    pub fn decode(&self) -> Cow<'_, str> {
        match &self.raw {
            Some(raw) => match text(raw) {
                Some(text) => unescape_text(text, self.escaper),
                None => unescape_with(raw, self.escaper),
            },
            None => {
                let joined = self.joined_bytes();
                Cow::Owned(unescape_with(&joined, self.escaper).into_owned())
            }
        }
    }

    /// Decode the whole value as its `,`-separated list, `;` kept literal.
    ///
    /// The list read to reach for: a comma separates the items of a list
    /// value, and nothing else in the value is cut.
    pub fn decode_list(&self) -> Vec<Cow<'_, str>> {
        let mut values = Vec::new();

        match &self.raw {
            Some(raw) => match text(raw) {
                Some(text) if memchr::memchr(b'\\', raw).is_none() => {
                    values.extend(text.split(',').map(Cow::Borrowed));
                }
                Some(text) => values.extend(
                    text_pieces(text, b',').map(|value| unescape_text(value, self.escaper)),
                ),
                None => split_on(raw, b',', |value| {
                    values.push(unescape_with(value, self.escaper))
                }),
            },
            None => {
                let joined = self.joined_bytes();
                split_on(&joined, b',', |value| {
                    values.push(Cow::Owned(unescape_with(value, self.escaper).into_owned()))
                });
            }
        }

        values
    }

    /// The whole value's `,`-separated items, still escaped.
    ///
    /// The raw twin of [`decode_list`](Self::decode_list), item for item. An
    /// unescape is not injective, `\N` and `\n` both reading as a line break
    /// (RFC 6350 3.4), so weighing what two sides wrote wants the bytes.
    pub(crate) fn raw_list(&self) -> Vec<Vec<u8>> {
        let mut values = Vec::new();

        match &self.raw {
            Some(raw) => split_on(raw, b',', |value| values.push(value.to_vec())),
            None => {
                let joined = self.joined_bytes();
                split_on(&joined, b',', |value| values.push(value.to_vec()));
            }
        }

        values
    }

    /// The whole value as raw unescaped bytes, not transcoded, for a value
    /// carrying a foreign charset. Its `;` and `,` stay literal.
    pub fn decode_bytes(&self) -> Cow<'_, [u8]> {
        match &self.raw {
            Some(raw) => unescape_bytes(raw, self.escaper),
            None => {
                let joined = self.joined_bytes();
                Cow::Owned(unescape_bytes(&joined, self.escaper).into_owned())
            }
        }
    }

    /// Decode the `i`th `;`-component as one string, its `,` kept literal.
    ///
    /// Only for a value the specification structures with `;`, which in vCard
    /// is `N`, `ADR`, `GENDER`, `ORG`, `GEO` and `CLIENTPIDMAP`; on any other,
    /// naming a component drops everything past the first `;`.
    pub fn decode_component(&self, i: usize) -> Cow<'_, str> {
        self.component_at(i)
            .map_or(Cow::Borrowed(""), |component| self.decode_whole(component))
    }

    /// Decode the `i`th `;`-component as its `,`-separated list.
    ///
    /// Carries the caveat of [`decode_component`](Self::decode_component): a
    /// value with no `;`-structure of its own wants
    /// [`decode_list`](Self::decode_list) instead.
    pub fn decode_component_list(&self, i: usize) -> Vec<Cow<'_, str>> {
        self.component_at(i)
            .map_or(Vec::new(), |component| self.decode_items(component))
    }

    /// Every `;`-component decoded as by
    /// [`decode_component`](Self::decode_component), in one pass rather than
    /// one walk from the start per index.
    pub(crate) fn decode_components(&self) -> impl Iterator<Item = Cow<'_, str>> {
        self.components()
            .map(|component| self.decode_whole(component))
    }

    /// Every `;`-component decoded as by
    /// [`decode_component_list`](Self::decode_component_list), in one pass.
    pub(crate) fn decode_component_lists(&self) -> impl Iterator<Item = Vec<Cow<'_, str>>> {
        self.components()
            .map(|component| self.decode_items(component))
    }

    /// A component as one string, its `,` kept literal.
    fn decode_whole<'s>(&'s self, component: Component<'s, 'a>) -> Cow<'s, str> {
        match component {
            // NOTE: The whole component slice already has the commas in place,
            // so unescaping it verbatim keeps them literal.
            Component::Text {
                text,
                escaped: false,
            } => Cow::Borrowed(text),
            Component::Text { text, .. } => unescape_text(text, self.escaper),
            Component::Raw(bytes) => unescape_with(bytes, self.escaper),
            Component::Split(leaves) => {
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
        }
    }

    /// A component as its `,`-separated list.
    fn decode_items<'s>(&'s self, component: Component<'s, 'a>) -> Vec<Cow<'s, str>> {
        match component {
            Component::Text {
                text,
                escaped: false,
            } => text.split(',').map(Cow::Borrowed).collect(),
            Component::Text { text, .. } => text_pieces(text, b',')
                .map(|value| unescape_text(value, self.escaper))
                .collect(),
            Component::Raw(bytes) => pieces(bytes, b',')
                .map(|value| unescape_with(value, self.escaper))
                .collect(),
            Component::Split(leaves) => leaves
                .iter()
                .map(|leaf| unescape_with(leaf.as_bytes(), self.escaper))
                .collect(),
        }
    }

    /// The `i`th `;`-component's `,`-separated items, still escaped.
    ///
    /// The raw twin of [`decode_component_list`](Self::decode_component_list),
    /// item for item, for the reason [`raw_list`](Self::raw_list) exists.
    pub(crate) fn raw_component_list(&self, i: usize) -> Vec<Vec<u8>> {
        let bytes = match self.component_at(i) {
            Some(Component::Text { text, .. }) => text.as_bytes(),
            Some(Component::Raw(bytes)) => bytes,
            Some(Component::Split(leaves)) => {
                return leaves.iter().map(|leaf| leaf.as_bytes().to_vec()).collect();
            }
            None => return Vec::new(),
        };

        let mut values = Vec::new();
        split_on(bytes, b',', |value| values.push(value.to_vec()));
        values
    }

    /// Wrap owned value bytes unsplit, so they go back on the wire exactly as
    /// given. Backs the value kinds that carry no escaping of their own.
    pub(crate) fn from_raw(bytes: Vec<u8>, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueNode {
            raw: Some(Cow::Owned(bytes)),
            components: Vec::new(),
            escaper,
        }
    }

    /// The raw (still-escaped) bytes of the first component's first value, for
    /// the simple single-value lines (the envelope values and diagnostics).
    pub(crate) fn first_value_bytes(&self) -> &[u8] {
        match &self.raw {
            Some(raw) => pieces(raw, b';').next().map_or(b"", first_value),
            None => self
                .components
                .first()
                .and_then(|leaves| leaves.first())
                .map_or(b"", |leaf| leaf.as_bytes()),
        }
    }

    /// Replace the whole value with one component of `,`-separated values,
    /// escaping each.
    ///
    /// The inverse of [`decode_list`](Self::decode_list), so reading a value
    /// and writing it back leaves no component of the old value behind.
    pub fn set<S: AsRef<str>>(&mut self, values: &[S]) {
        self.raw = None;
        self.components.clear();
        self.components.push(encode_component(values, self.escaper));
    }

    /// Replace the whole value with one component of raw value bytes (the
    /// foreign-charset escape hatch), escaping structural separators but
    /// writing the bytes verbatim.
    pub fn set_bytes<B: AsRef<[u8]>>(&mut self, values: &[B]) {
        self.raw = None;
        self.components.clear();
        self.components
            .push(encode_bytes_component(values, self.escaper));
    }

    /// Set the `i`th component, escaping each value and padding with empty
    /// components when needed. For a `;`-structured value, as
    /// [`decode_component`](Self::decode_component) spells out.
    pub fn set_component<S: AsRef<str>>(&mut self, i: usize, values: &[S]) {
        self.materialize();

        while self.components.len() <= i {
            self.components.push(Vec::new());
        }

        self.components[i] = encode_component(values, self.escaper);
    }

    /// Set the `i`th component from raw value bytes (the foreign-charset escape
    /// hatch), escaping structural separators but writing the bytes verbatim.
    pub fn set_component_bytes<B: AsRef<[u8]>>(&mut self, i: usize, values: &[B]) {
        self.materialize();

        while self.components.len() <= i {
            self.components.push(Vec::new());
        }

        self.components[i] = encode_bytes_component(values, self.escaper);
    }

    /// The number of `,`-separated values in the `i`th component (zero when the
    /// component does not exist).
    pub fn value_count(&self, i: usize) -> usize {
        match self.component_at(i) {
            Some(Component::Text { text, .. }) => pieces(text.as_bytes(), b',').count(),
            Some(Component::Raw(bytes)) => pieces(bytes, b',').count(),
            Some(Component::Split(leaves)) => leaves.len(),
            None => 0,
        }
    }

    /// Replace the `j`th value of the `i`th component in place, re-escaping
    /// only that leaf. Pads with empty values when `j` is past the end.
    pub fn set_value_at<S: AsRef<str>>(&mut self, i: usize, j: usize, value: S) {
        let escaper = self.escaper;
        let component = self.component_mut(i);

        while component.len() <= j {
            component.push(encode_leaf("", escaper));
        }

        component[j] = encode_leaf(value, escaper);
    }

    /// Insert a value at position `j` of the `i`th component (clamped to the
    /// end), escaping only the new leaf.
    pub fn insert_value_at<S: AsRef<str>>(&mut self, i: usize, j: usize, value: S) {
        let escaper = self.escaper;
        let component = self.component_mut(i);
        let at = j.min(component.len());

        component.insert(at, encode_leaf(value, escaper));
    }

    /// Append a value to the `i`th component, escaping only the new leaf.
    pub fn push_value<S: AsRef<str>>(&mut self, i: usize, value: S) {
        let escaper = self.escaper;
        let component = self.component_mut(i);

        component.push(encode_leaf(value, escaper));
    }

    /// Remove the `j`th value of the `i`th component, splicing it out; a no-op
    /// when either index is out of range.
    pub fn remove_value_at(&mut self, i: usize, j: usize) {
        self.materialize();

        if let Some(component) = self.components.get_mut(i)
            && j < component.len()
        {
            component.remove(j);
        }
    }

    /// The value's bytes while it is unsplit, as [`write_bytes`] writes them.
    ///
    /// [`write_bytes`]: Self::write_bytes
    pub(crate) fn unsplit_bytes(&self) -> Option<&[u8]> {
        self.raw.as_deref()
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

    /// The whole value's still-escaped bytes, reassembled from the split
    /// components. Only a split node needs this: an unsplit one already holds
    /// the value as one slice and reads it borrowed.
    fn joined_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_bytes(&mut out);
        out
    }

    /// Locate the `i`th component, as a slice of the unsplit value or as the
    /// already-split leaves.
    fn component_at(&self, i: usize) -> Option<Component<'_, 'a>> {
        self.components().nth(i)
    }

    /// Every component in order.
    ///
    /// An unsplit value is checked for UTF-8 and searched for a backslash
    /// once, here: its components and their items are then slices of text
    /// needing no check of their own, cut at every separator when no escape
    /// can hide one.
    fn components(&self) -> Components<'_, 'a> {
        let Some(raw) = &self.raw else {
            return Components::Split(self.components.iter());
        };

        match text(raw) {
            Some(text) if memchr::memchr(b'\\', raw).is_none() => {
                Components::Plain(text.split(';'))
            }
            Some(text) => Components::Escaped {
                text,
                pieces: pieces(raw, b';'),
            },
            None => Components::Raw(pieces(raw, b';')),
        }
    }

    /// Borrow the `i`th component's leaves for in-place editing, splitting the
    /// value first and padding with empty components up to `i`.
    fn component_mut(&mut self, i: usize) -> &mut Vec<VcardValueLeaf<'a>> {
        self.materialize();

        while self.components.len() <= i {
            self.components.push(Vec::new());
        }

        &mut self.components[i]
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
    /// The component's text, still `,`-joined, borrowed from an unsplit value
    /// that is UTF-8.
    Text {
        /// The text, still escaped.
        text: &'s str,
        /// Whether the value holds a backslash, which may escape a `,`.
        escaped: bool,
    },
    /// The component's bytes, still `,`-joined, borrowed from an unsplit
    /// value that is not UTF-8.
    Raw(&'s [u8]),
    /// The component's already-split leaves.
    Split(&'s [VcardValueLeaf<'a>]),
}

/// The iterator behind [`components`](VcardValueNode::components).
enum Components<'s, 'a> {
    /// A UTF-8 value with no backslash, cut at every `;`.
    Plain(str::Split<'s, char>),
    /// A UTF-8 value whose backslashes may escape a `;`.
    Escaped { text: &'s str, pieces: Pieces<'s> },
    /// A value that is not UTF-8.
    Raw(Pieces<'s>),
    /// The leaves of a split value.
    Split(slice::Iter<'s, Vec<VcardValueLeaf<'a>>>),
}

impl<'s, 'a> Iterator for Components<'s, 'a> {
    type Item = Component<'s, 'a>;

    fn next(&mut self) -> Option<Component<'s, 'a>> {
        match self {
            Self::Plain(split) => split.next().map(|text| Component::Text {
                text,
                escaped: false,
            }),
            Self::Escaped { text, pieces } => {
                let text = *text;

                pieces.span().map(|span| Component::Text {
                    text: &text[span],
                    escaped: true,
                })
            }
            Self::Raw(pieces) => pieces.next().map(Component::Raw),
            Self::Split(leaves) => leaves.next().map(|leaves| Component::Split(leaves)),
        }
    }
}

/// The first `,`-separated value of a component (escape-aware): the bytes up to
/// the first unescaped comma, or the whole component when it has none.
fn first_value(component: &[u8]) -> &[u8] {
    pieces(component, b',').next().unwrap_or(component)
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

/// Call `piece` for each span between unescaped `sep` bytes, at least once.
///
/// A backslash escapes the next byte, so `\;` and `\,` do not split, and
/// `memchr` skips straight to the next `sep` or backslash, so a large
/// separator-free value (base64, say) passes in one go.
fn split_on<'b>(bytes: &'b [u8], sep: u8, piece: impl FnMut(&'b [u8])) {
    pieces(bytes, sep).for_each(piece);
}

/// The pieces [`split_on`] hands out, as an iterator, so a reader after one
/// of them stops there.
fn pieces(bytes: &[u8], sep: u8) -> Pieces<'_> {
    Pieces {
        bytes,
        sep,
        start: Some(0),
    }
}

/// The pieces of a UTF-8 value, as [`pieces`] cuts its bytes: every cut sits
/// at an ASCII separator, so each piece is text too.
fn text_pieces(text: &str, sep: u8) -> impl Iterator<Item = &str> {
    let mut pieces = pieces(text.as_bytes(), sep);
    iter::from_fn(move || pieces.span().map(|span| &text[span]))
}

/// The iterator behind [`pieces`].
struct Pieces<'b> {
    bytes: &'b [u8],
    sep: u8,
    /// Where the next piece starts, `None` once the last one went out.
    start: Option<usize>,
}

impl Pieces<'_> {
    /// Where the next piece sits in the bytes.
    fn span(&mut self) -> Option<Range<usize>> {
        let start = self.start?;
        let mut i = start;

        while let Some(offset) = memchr::memchr2(b'\\', self.sep, &self.bytes[i..]) {
            let pos = i + offset;
            if self.bytes[pos] == b'\\' {
                i = (pos + 2).min(self.bytes.len());
            } else {
                self.start = Some(pos + 1);
                return Some(start..pos);
            }
        }

        self.start = None;
        Some(start..self.bytes.len())
    }
}

impl<'b> Iterator for Pieces<'b> {
    type Item = &'b [u8];

    fn next(&mut self) -> Option<&'b [u8]> {
        let bytes = self.bytes;
        self.span().map(|span| &bytes[span])
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

    use crate::tree::value::node::VcardValueNode;

    #[test]
    fn splits_components_and_values_then_round_trips() {
        let node = VcardValueNode::parse(b"a;b,c;");
        assert_eq!(node.component_count(), 3);
        assert_eq!(
            node.decode_component_list(1),
            vec![Cow::Borrowed("b"), Cow::Borrowed("c")]
        );
        assert_eq!(node.to_string(), "a;b,c;");
    }

    #[test]
    fn keeps_escaped_separators_inside_one_value() {
        let node = VcardValueNode::parse(br"a\,b\;c;d");
        assert_eq!(node.component_count(), 2);
        assert_eq!(node.decode_component_list(0).len(), 1);
        assert_eq!(node.to_string(), r"a\,b\;c;d");
    }

    #[test]
    fn an_edit_splits_and_preserves_untouched_components() {
        let mut node = VcardValueNode::parse(b"a;b;c");
        node.set_component(1, &["X"]);
        assert_eq!(node.to_string(), "a;X;c");
    }

    /// Every reader answers the same before and after the node materializes.
    ///
    /// A node holds raw bytes until the first edit splits it into components,
    /// and each reader has a branch per state. A disagreement would surface as
    /// a value changing shape the moment an unrelated component is written.
    fn assert_readers_agree(node: &VcardValueNode<'_>, whole: &str) {
        assert_eq!(node.component_count(), whole.split(';').count());
        assert_eq!(node.value_count(1), 2);
        assert_eq!(node.decode(), whole);
        assert_eq!(node.decode_bytes().as_ref(), whole.as_bytes());
        assert_eq!(
            node.decode_list(),
            whole.split(',').map(Cow::Borrowed).collect::<Vec<_>>(),
        );
        assert_eq!(node.decode_component(0), "a");
        assert_eq!(
            node.decode_component_list(1),
            vec![Cow::Borrowed("b"), Cow::Borrowed("c")],
        );
        assert_eq!(node.decode_component(1), "b,c");
        assert_eq!(node.decode_component_list(2), vec![Cow::Borrowed("d")]);
    }

    #[test]
    fn readers_agree_before_and_after_an_edit_materializes_the_node() {
        let mut node = VcardValueNode::parse(b"a;b,c;d");
        assert_readers_agree(&node, "a;b,c;d");

        // NOTE: Writing component 3 leaves the read components alone, but it is
        // what moves the node off its raw bytes.
        node.set_component(3, &["e"]);
        assert_readers_agree(&node, "a;b,c;d;e");
        assert_eq!(node.to_string(), "a;b,c;d;e");
    }

    #[test]
    fn readers_agree_after_an_owned_node_is_edited() {
        let mut node = VcardValueNode::parse(b"a;b,c;d").into_static();
        assert_readers_agree(&node, "a;b,c;d");

        node.set_component(3, &["e"]);
        assert_readers_agree(&node, "a;b,c;d;e");
        assert_eq!(node.to_string(), "a;b,c;d;e");
    }

    /// The un-indexed writers are the inverse of the un-indexed readers: a
    /// value read whole and written back must come back as it went in, not
    /// gain a tail from the components it replaced.
    #[test]
    fn a_whole_value_write_leaves_no_component_of_the_old_value_behind() {
        let mut node = VcardValueNode::parse(b"a;b,c;d");
        let whole = node.decode().into_owned();
        node.set(&[whole]);
        assert_eq!(node.decode(), "a;b,c;d");
        assert_eq!(node.to_string(), r"a\;b\,c\;d");

        let mut node = VcardValueNode::parse(b"a;b,c;d");
        let whole = node.decode_bytes().into_owned();
        node.set_bytes(&[whole]);
        assert_eq!(node.decode_bytes().as_ref(), b"a;b,c;d");
    }
}
