//! # Value cursor
//!
//! The generic in-place edit cursor used by every property lens but `N` and
//! `ADR`.
//!
//! A cursor borrows a content line mutably and lets you read and write its
//! value through the codec: getters cook (unescape, QP-decode); the text setter
//! writes literally (a simple 2.1 value is not compound) while the per-component
//! setters escape, writing through to the syntax node. A per-component setter
//! only rewrites the component it touches, so every other leaf (and every
//! parameter) of a parsed line stays byte for byte intact. In vCard 2.1 a value
//! is a flat list of single-valued `;`-components, so [`VcardValueCursor`]
//! exposes a single-text convenience plus per-component access for the structured
//! kinds (`ORG`); the bespoke
//! [`NCursor`](crate::v21::tree::prop::n::NCursor) and
//! [`AdrCursor`](crate::v21::tree::prop::adr::AdrCursor) name `N`'s and `ADR`'s
//! components.

use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

use crate::v21::tree::{encode::escape, leaf::VcardLeaf, line::VcardLine, param::VcardParamLens};

/// A typed cursor over a content line's value, editing in place and byte
/// preserving for the components it does not touch.
pub struct VcardValueCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl VcardValueCursor<'_, '_> {
    /// The whole value as a single decoded text (non-compound: the `;` is
    /// literal). For the structured kinds use [`component`](Self::component).
    pub fn text(&self) -> Cow<'_, str> {
        self.line.cooked_value()
    }

    /// Set the value to a single text, written literally (a 2.1 simple value is
    /// not compound, so its `;` is not escaped). Replaces every component.
    pub fn set_text(&mut self, value: impl AsRef<str>) {
        self.line.value.components = vec![VcardLeaf::from(value.as_ref().to_string())];
    }

    /// The `i`th component, decoded, for structured values.
    pub fn component(&self, i: usize) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(i).unwrap_or_default()
    }

    /// Set the `i`th component, escaping it and preserving the rest.
    pub fn set_component(&mut self, i: usize, value: impl AsRef<str>) {
        self.line.value.set_at(i, value.as_ref());
    }

    /// All the value's components, decoded.
    pub fn components(&self) -> Vec<Cow<'_, str>> {
        self.line.cooked()
    }

    /// Replace every component, escaping each value.
    pub fn set_components<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.components = values
            .iter()
            .map(|v| VcardLeaf::from(escape(v.as_ref()).into_owned()))
            .collect();
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::v21::tree::{
        cst::VcardCst,
        prop::{r#fn::FN, org::ORG},
    };

    #[test]
    fn edits_a_scalar_value_in_place_writing_it_literally() {
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nFN:John\r\nEND:VCARD\r\n").unwrap();
        card.prop_mut::<FN>().unwrap().set_text("Jane; Q");
        assert!(card.to_string().contains("FN:Jane; Q\r\n"));
    }

    #[test]
    fn replaces_all_components_escaping_each() {
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nORG:Old\r\nEND:VCARD\r\n").unwrap();
        card.prop_mut::<ORG>().unwrap().set_components(&["A", "B"]);
        assert!(card.to_string().contains("ORG:A;B\r\n"));
    }
}
