//! # Value cursor
//!
//! The generic in-place edit cursor used by every property lens but `N`.
//!
//! A cursor borrows a content line mutably and lets you read and write its
//! value through the codec: getters decode (unescape), setters encode (escape)
//! and write through to the syntax node. Crucially, a setter only rewrites the
//! component it touches, so every other leaf (and every parameter) of a parsed
//! line stays byte for byte intact. [`VcardValueCursor`] exposes both
//! convenience accessors for the common single-value and list shapes and raw
//! component-level access for the structured kinds (`ADR`, `GENDER`, `ORG`,
//! `CLIENTPIDMAP`); the bespoke
//! [`VcardNCursor`](crate::tree::prop::n::VcardNCursor) names `N`'s components.
//!
//! Beside the UTF-8 text accessors it offers a raw byte hatch
//! ([`bytes`](VcardValueCursor::bytes) /
//! [`set_bytes`](VcardValueCursor::set_bytes)) for a value in a foreign
//! charset, and, behind the content-encoding features, the
//! [`quoted_printable`](VcardValueCursor::quoted_printable) and
//! [`charset`](VcardValueCursor::charset) decoders.

use alloc::{borrow::Cow, vec::Vec};

use crate::tree::{line::VcardLine, param::lens::VcardParamLens, value::node::VcardValueNode};

/// A typed cursor over a content line's value, editing in place and byte
/// preserving for the components it does not touch.
pub struct VcardValueCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl<'a> VcardValueCursor<'_, 'a> {
    /// The whole value as a single decoded text (component 0, value 0).
    pub fn text(&self) -> Cow<'_, str> {
        self.line.value.decode_scalar_at(0)
    }

    /// Set the value to a single text, escaping and preserving any other
    /// components. Writes UTF-8; to keep a foreign charset, transcode yourself
    /// and use [`set_bytes`](Self::set_bytes).
    pub fn set_text(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(0, &[value]);
    }

    /// The whole value's raw bytes (component 0, value 0), unescaped but not
    /// transcoded and not transfer-decoded, for a value carrying a foreign
    /// charset. To resolve `QUOTED-PRINTABLE` or a `CHARSET`, use the
    /// [`quoted_printable`](Self::quoted_printable) /
    /// [`charset`](Self::charset) feature helpers.
    pub fn bytes(&self) -> Cow<'_, [u8]> {
        self.line.value.decode_bytes_at(0)
    }

    /// Set the value to raw bytes (the foreign-charset escape hatch), escaping
    /// structural separators but writing the bytes verbatim and preserving any
    /// other components. The card's `CHARSET` parameter is left untouched: it
    /// is the caller's to keep consistent.
    pub fn set_bytes(&mut self, value: impl AsRef<[u8]>) {
        self.line.value.set_bytes_at(0, &[value]);
    }

    /// Decode the value's `QUOTED-PRINTABLE` `=XX` octets to raw bytes when the
    /// line declares that encoding, else the raw [`bytes`](Self::bytes). Still
    /// in the value's own (possibly foreign) charset; pair with
    /// [`charset`](Self::charset) to get text. Requires the `quoted-printable`
    /// feature.
    #[cfg(feature = "quoted-printable")]
    pub fn quoted_printable(&self) -> Vec<u8> {
        let raw = self.bytes();

        if self.line.is_quoted_printable() {
            quoted_printable::decode(raw.as_ref(), quoted_printable::ParseMode::Robust)
                .unwrap_or_else(|_| raw.into_owned())
        } else {
            raw.into_owned()
        }
    }

    /// Transcode the value to text using its `CHARSET` parameter (defaulting to
    /// UTF-8 when absent or unrecognised). When the `quoted-printable` feature
    /// is also on, `QUOTED-PRINTABLE` octets are resolved first. Requires the
    /// `encoding` feature.
    #[cfg(feature = "encoding")]
    pub fn charset(&self) -> alloc::string::String {
        #[cfg(feature = "quoted-printable")]
        let bytes = self.quoted_printable();
        #[cfg(not(feature = "quoted-printable"))]
        let bytes = self.bytes().into_owned();

        let encoding = self
            .line
            .charset_label()
            .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()))
            .unwrap_or(encoding_rs::UTF_8);

        encoding.decode_without_bom_handling(&bytes).0.into_owned()
    }

    /// The value's first component as a decoded list (its `,`-separated
    /// values).
    pub fn list(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(0)
    }

    /// Set the value's first component to a list, escaping each value.
    pub fn set_list<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(0, values);
    }

    /// The `i`th component as a decoded list, for structured values.
    pub fn component(&self, i: usize) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(i)
    }

    /// Set the `i`th component, escaping each value and preserving the rest.
    pub fn set_component<S: AsRef<str>>(&mut self, i: usize, values: &[S]) {
        self.line.value.set_at(i, values);
    }

    /// Walk into the `i`th component to edit its `,`-separated values one at a
    /// time, splicing a single leaf per edit and leaving the siblings' bytes
    /// exactly as parsed.
    pub fn list_at(&mut self, i: usize) -> VcardListCursor<'_, 'a> {
        VcardListCursor {
            node: &mut self.line.value,
            component: i,
        }
    }

    /// Walk into the first component's list (the flat `,`-list shape), the
    /// common case of [`list_at`](Self::list_at).
    pub fn list_mut(&mut self) -> VcardListCursor<'_, 'a> {
        self.list_at(0)
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}

/// A cursor over one component's `,`-separated values, editing them per item.
///
/// Every mutation touches a single leaf (or splices one in or out) and
/// re-emits only the structural separators, so an untouched value keeps the
/// exact bytes it was parsed with, escaping and all. Obtained from
/// [`VcardValueCursor::list_at`] / [`list_mut`](VcardValueCursor::list_mut).
pub struct VcardListCursor<'c, 'a> {
    node: &'c mut VcardValueNode<'a>,
    component: usize,
}

impl VcardListCursor<'_, '_> {
    /// The number of values in the walked component.
    pub fn len(&self) -> usize {
        self.node.value_count(self.component)
    }

    /// Whether the walked component has no values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The `j`th value, decoded, or `None` when the index is out of range.
    pub fn get(&self, j: usize) -> Option<Cow<'_, str>> {
        self.node.decode_at(self.component).into_iter().nth(j)
    }

    /// Replace the `j`th value in place, re-escaping only that leaf.
    pub fn set<S: AsRef<str>>(&mut self, j: usize, value: S) -> &mut Self {
        self.node.set_value_at(self.component, j, value);
        self
    }

    /// Insert a value at position `j` (clamped to the end), escaping only the
    /// new leaf.
    pub fn insert<S: AsRef<str>>(&mut self, j: usize, value: S) -> &mut Self {
        self.node.insert_value_at(self.component, j, value);
        self
    }

    /// Append a value, escaping only the new leaf.
    pub fn push<S: AsRef<str>>(&mut self, value: S) -> &mut Self {
        self.node.push_value(self.component, value);
        self
    }

    /// Remove the `j`th value, splicing it out; a no-op when out of range.
    pub fn remove(&mut self, j: usize) -> &mut Self {
        self.node.remove_value_at(self.component, j);
        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::{
        cst::VcardCst,
        prop::{adr::ADR, r#fn::FN},
    };

    #[test]
    fn edits_a_scalar_value_in_place_escaping_it() {
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John\r\nEND:VCARD\r\n").unwrap();
        card.prop_mut::<FN>().unwrap().set_text("Jane, Q");
        assert!(card.to_string().contains("FN:Jane\\, Q\r\n"));
    }

    #[test]
    fn writes_and_reads_a_foreign_charset_value_as_raw_bytes() {
        use crate::tree::prop::note::NOTE;

        let mut card = VcardCst::parse(
            "BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE;CHARSET=ISO-8859-1:x\r\nEND:VCARD\r\n",
        )
        .unwrap();

        // NOTE: "café" in ISO-8859-1: the trailing 0xE9 is not valid UTF-8.
        let latin1 = [b'c', b'a', b'f', 0xE9];
        card.prop_mut::<NOTE>().unwrap().set_bytes(latin1);

        assert_eq!(card.prop_mut::<NOTE>().unwrap().bytes().as_ref(), &latin1);
        assert!(card.to_bytes().windows(4).any(|window| window == latin1));
    }

    #[test]
    fn bytes_returns_the_raw_undecoded_value() {
        use crate::tree::prop::note::NOTE;

        // NOTE: Core does not resolve QP: bytes() is the raw wire value.
        let mut card = VcardCst::parse(concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:2.1\r\n",
            "NOTE;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:caf=E9\r\n",
            "END:VCARD\r\n",
        ))
        .unwrap();

        assert_eq!(card.prop_mut::<NOTE>().unwrap().bytes().as_ref(), b"caf=E9");
    }

    #[cfg(feature = "quoted-printable")]
    #[test]
    fn quoted_printable_helper_resolves_octets() {
        use crate::tree::prop::note::NOTE;

        // NOTE: =E9 is the Latin-1 'é' octet; the helper resolves QP to raw
        // bytes.
        let mut card = VcardCst::parse(concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:2.1\r\n",
            "NOTE;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:caf=E9\r\n",
            "END:VCARD\r\n",
        ))
        .unwrap();

        assert_eq!(
            card.prop_mut::<NOTE>().unwrap().quoted_printable(),
            [b'c', b'a', b'f', 0xE9],
        );
    }

    #[cfg(all(feature = "encoding", feature = "quoted-printable"))]
    #[test]
    fn charset_helper_transcodes_to_utf8() {
        use crate::tree::prop::note::NOTE;

        // NOTE: ISO-8859-1 + quoted-printable "café": the charset helper
        // (composing the QP helper) yields the UTF-8 string.
        let mut card = VcardCst::parse(concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:2.1\r\n",
            "NOTE;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:caf=E9\r\n",
            "END:VCARD\r\n",
        ))
        .unwrap();

        assert_eq!(card.prop_mut::<NOTE>().unwrap().charset(), "café");
    }

    #[test]
    fn edits_one_structured_component_preserving_the_rest() {
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nADR:;;Old St;;;;\r\nEND:VCARD\r\n")
                .unwrap();
        card.prop_mut::<ADR>().unwrap().set_street(&["New St"]);
        assert!(card.to_string().contains("ADR:;;New St;;;;\r\n"));
    }

    #[test]
    fn walks_a_list_editing_items_without_reformatting_siblings() {
        use crate::tree::prop::nickname::NICKNAME;

        // NOTE: `a\:b` is a redundant (non-canonical) escape: `:` need not be
        // escaped. A whole-list rewrite would decode and normalise it to `a:b`;
        // a per-item edit of a *different* value must leave it byte for byte.
        let mut card = VcardCst::parse(concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "NICKNAME:a\\:b,middle,z\r\n",
            "END:VCARD\r\n",
        ))
        .unwrap();

        card.prop_mut::<NICKNAME>().unwrap().list_mut().remove(1);

        // NOTE: The surviving `a\:b` keeps its bytes; only `middle` and its
        // comma go.
        let out = card.to_string();
        assert!(out.contains("NICKNAME:a\\:b,z\r\n"), "got: {out}");
    }

    #[test]
    fn walks_a_list_setting_inserting_and_pushing() {
        use crate::tree::prop::nickname::NICKNAME;

        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nNICKNAME:a,b,c\r\nEND:VCARD\r\n")
                .unwrap();

        {
            let mut cursor = card.prop_mut::<NICKNAME>().unwrap();
            let mut list = cursor.list_mut();
            assert_eq!(list.len(), 3);
            assert_eq!(list.get(1).as_deref(), Some("b"));

            list.set(1, "B").insert(0, "first").push("last");
        }

        let out = card.to_string();
        assert!(out.contains("NICKNAME:first,a,B,c,last\r\n"), "got: {out}");
    }

    #[test]
    fn exercises_every_generic_accessor() {
        use crate::tree::prop::note::NOTE;

        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:a,b\r\nEND:VCARD\r\n").unwrap();
        let mut cursor = card.prop_mut::<NOTE>().unwrap();

        let _ = cursor.text();
        let _ = cursor.list();
        let _ = cursor.component(0);
        cursor.set_text("x");
        cursor.set_list(&["a", "b"]);
        cursor.set_component(1, &["y"]);
    }
}
