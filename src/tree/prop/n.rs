//! # N lens
//!
//! The `N` property lens and its bespoke edit cursor.
//!
//! `N` is the showcase of the structured-value path: unlike the macro-generated
//! lenses in the parent module, it pairs [`VcardN`] with a dedicated
//! [`NCursor`] that names the five components, so callers write
//! `cursor.set_family(...)` rather than `cursor.set_component(0, ...)`. The
//! decode/encode projections still delegate to [`VcardN`]'s inherent methods (in
//! [`crate::tree::decode`] / [`crate::tree::encode`]); only the cursor is
//! bespoke. Edits are byte preserving: writing one component leaves the others,
//! and every parameter, untouched.

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    prop::VCARD_N,
    tree::{
        lens::{VcardParamLens, VcardPropLens},
        line::VcardLine,
        value::VcardValueNode,
    },
    value::n::VcardN,
};

/// The `N` property lens.
pub struct N;

impl VcardPropLens for N {
    const NAME: &'static str = VCARD_N;

    type Target<'v> = VcardN<'v>;

    type Cursor<'c, 'a>
        = NCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardN<'v> {
        VcardN::decode(value)
    }

    fn encode(decoded: &VcardN<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> NCursor<'c, 'a> {
        NCursor { line }
    }
}

/// A typed cursor over an N line: getters decode, setters encode and write
/// through to the syntax node, leaving every untouched component (and every
/// parameter) byte for byte intact.
pub struct NCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl NCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardN<'_> {
        VcardN::decode(&self.line.value)
    }

    /// The family names, decoded.
    pub fn family(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(0)
    }

    /// The given names, decoded.
    pub fn given(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(1)
    }

    /// The additional names, decoded.
    pub fn additional(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(2)
    }

    /// The honorific prefixes, decoded.
    pub fn prefixes(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(3)
    }

    /// The honorific suffixes, decoded.
    pub fn suffixes(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(4)
    }

    /// Set the family names, escaping and preserving the rest of the line.
    pub fn set_family<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(0, values);
    }

    /// Set the given names, escaping and preserving the rest of the line.
    pub fn set_given<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(1, values);
    }

    /// Set the additional names, escaping and preserving the rest of the line.
    pub fn set_additional<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(2, values);
    }

    /// Set the honorific prefixes, escaping and preserving the rest of the line.
    pub fn set_prefixes<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(3, values);
    }

    /// Set the honorific suffixes, escaping and preserving the rest of the line.
    pub fn set_suffixes<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(4, values);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::tree::{cst::VcardCst, prop::n::N};

    #[test]
    fn names_components_and_sets_them_preserving_the_rest() {
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nN:Doe;John;;;\r\nEND:VCARD\r\n")
                .unwrap();

        assert_eq!(
            card.prop_mut::<N>().unwrap().family(),
            vec![Cow::Borrowed("Doe")],
        );

        card.prop_mut::<N>().unwrap().set_given(&["Jane"]);
        assert!(card.to_string().contains("N:Doe;Jane;;;\r\n"));
    }
}
