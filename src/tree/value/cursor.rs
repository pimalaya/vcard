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
    /// components.
    pub fn set_text(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(0, &[value]);
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
    fn edits_one_structured_component_preserving_the_rest() {
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nADR:;;Old St;;;;\r\nEND:VCARD\r\n")
                .unwrap();
        card.prop_mut::<ADR>().unwrap().set_street(&["New St"]);
        assert!(card.to_string().contains("ADR:;;New St;;;;\r\n"));
    }
}
