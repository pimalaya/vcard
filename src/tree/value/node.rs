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

use core::fmt;

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::tree::{
    codec::{
        encode::{encode_bytes_component, encode_component, encode_leaf},
        mode::VcardEscaper,
        unescape::{unescape_bytes, unescape_with},
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
        Self {
            raw: Some(Cow::Borrowed(value)),
            components: Vec::new(),
            escaper: VcardEscaper::default(),
        }
    }

    /// Build a value node from already-split components (the model encode
    /// path); there is no `raw`, so the components are the source of truth.
    pub(crate) fn from_components(
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
            Some(raw) => {
                let mut count = 0;
                split_on(raw, b';', |_| count += 1);
                count
            }
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
            Some(raw) => unescape_with(raw, self.escaper),
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
            Some(raw) => split_on(raw, b',', |value| {
                values.push(unescape_with(value, self.escaper))
            }),
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
        match self.component_at(i) {
            // NOTE: The whole component slice already has the commas in place,
            // so unescaping it verbatim keeps them literal.
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

    /// Decode the `i`th `;`-component as its `,`-separated list.
    ///
    /// Carries the caveat of [`decode_component`](Self::decode_component): a
    /// value with no `;`-structure of its own wants
    /// [`decode_list`](Self::decode_list) instead.
    pub fn decode_component_list(&self, i: usize) -> Vec<Cow<'_, str>> {
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

    /// The `i`th `;`-component's `,`-separated items, still escaped.
    ///
    /// The raw twin of [`decode_component_list`](Self::decode_component_list),
    /// item for item, for the reason [`raw_list`](Self::raw_list) exists.
    pub(crate) fn raw_component_list(&self, i: usize) -> Vec<Vec<u8>> {
        match self.component_at(i) {
            Some(Component::Raw(bytes)) => {
                let mut values = Vec::new();
                split_on(bytes, b',', |value| values.push(value.to_vec()));
                values
            }
            Some(Component::Split(leaves)) => {
                leaves.iter().map(|leaf| leaf.as_bytes().to_vec()).collect()
            }
            None => Vec::new(),
        }
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
        match self.component_at(0) {
            Some(Component::Raw(bytes)) => first_value(bytes),
            Some(Component::Split(leaves)) => {
                leaves.first().map(|leaf| leaf.as_bytes()).unwrap_or(b"")
            }
            None => b"",
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
            Some(Component::Raw(bytes)) => {
                let mut count = 0;
                split_on(bytes, b',', |_| count += 1);
                count
            }
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
    /// The component's bytes, still `,`-joined, borrowed from the unsplit
    /// value.
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

/// Call `piece` for each span between unescaped `sep` bytes, at least once.
///
/// A backslash escapes the next byte, so `\;` and `\,` do not split, and
/// `memchr` skips straight to the next `sep` or backslash, so a large
/// separator-free value (base64, say) passes in one go.
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

    #[test]
    fn a_whole_value_write_leaves_no_component_of_the_old_value_behind() {
        // NOTE: The un-indexed writers are the inverse of the un-indexed
        // readers: a value read whole and written back must come back as it
        // went in, not gain a tail from the components it replaced.
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
