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
//! `CLIENTPIDMAP`); the bespoke [`VcardNCursor`](crate::tree::prop::n::VcardNCursor)
//! names `N`'s components.

use alloc::{borrow::Cow, vec::Vec};

use crate::tree::{line::VcardLine, param::VcardParamLens};

/// A typed cursor over a content line's value, editing in place and byte
/// preserving for the components it does not touch.
pub struct VcardValueCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl VcardValueCursor<'_, '_> {
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
    /// other components. The card's `CHARSET` parameter is left untouched: it is
    /// the caller's to keep consistent.
    pub fn set_bytes(&mut self, value: impl AsRef<[u8]>) {
        self.line.value.set_bytes_at(0, &[value]);
    }

    /// Decode the value's `QUOTED-PRINTABLE` `=XX` octets to raw bytes when the
    /// line declares that encoding, else the raw [`bytes`](Self::bytes). Still in
    /// the value's own (possibly foreign) charset; pair with
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
    /// UTF-8 when absent or unrecognised). When the `quoted-printable` feature is
    /// also on, `QUOTED-PRINTABLE` octets are resolved first. Requires the
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

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
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

        // "café" in ISO-8859-1: the trailing 0xE9 is not valid UTF-8.
        let latin1 = [b'c', b'a', b'f', 0xE9];
        card.prop_mut::<NOTE>().unwrap().set_bytes(latin1);

        assert_eq!(card.prop_mut::<NOTE>().unwrap().bytes().as_ref(), &latin1);
        assert!(card.to_bytes().windows(4).any(|window| window == latin1));
    }

    #[test]
    fn bytes_returns_the_raw_undecoded_value() {
        use crate::tree::prop::note::NOTE;

        // Core does not resolve QP: bytes() is the raw wire value.
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

        // =E9 is the Latin-1 'é' octet; the helper resolves QP to raw bytes.
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

        // ISO-8859-1 + quoted-printable "café": the charset helper (composing the
        // QP helper) yields the UTF-8 string.
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
}
