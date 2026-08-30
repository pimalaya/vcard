//! # N lens
//!
//! Reading and editing the `N` property in place through a bespoke cursor.
//!
//! `N` is the showcase of the structured-value path: it pairs [`VcardN`] with
//! a dedicated [`VcardNCursor`] naming the five components, so callers write
//! `cursor.set_family(...)` rather than `cursor.set_component(0, ...)`.
//!
//! Only the cursor is bespoke: decode and encode use the lens defaults, which
//! delegate to [`VcardN`]'s [`VcardCodec`]
//! impl in [`crate::tree::value`].
//!
//! Its RFC contract sits on the marker, [`N`].

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    prop::n::N,
    tree::{
        codec::VcardCodec, line::VcardLine, param::lens::VcardParamLens, prop::lens::VcardPropLens,
    },
    value::n::VcardN,
};

impl VcardPropLens for N {
    type Target<'v> = VcardN<'v>;

    type Cursor<'c, 'a>
        = VcardNCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardNCursor<'c, 'a> {
        VcardNCursor { line }
    }
}

/// A typed cursor over an N line: getters decode, setters encode and write
/// through to the syntax node, leaving every untouched component (and every
/// parameter) byte for byte intact.
pub struct VcardNCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl VcardNCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardN<'_> {
        VcardN::decode(&self.line.value)
    }

    /// The family names, decoded.
    pub fn family(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_component_list(0)
    }

    /// The given names, decoded.
    pub fn given(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_component_list(1)
    }

    /// The additional names, decoded.
    pub fn additional(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_component_list(2)
    }

    /// The honorific prefixes, decoded.
    pub fn prefixes(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_component_list(3)
    }

    /// The honorific suffixes, decoded.
    pub fn suffixes(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_component_list(4)
    }

    /// Set the family names, escaping and preserving the rest of the line.
    pub fn set_family<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_component(0, values);
    }

    /// Set the given names, escaping and preserving the rest of the line.
    pub fn set_given<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_component(1, values);
    }

    /// Set the additional names, escaping and preserving the rest of the line.
    pub fn set_additional<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_component(2, values);
    }

    /// Set the honorific prefixes, escaping and preserving the rest of the
    /// line.
    pub fn set_prefixes<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_component(3, values);
    }

    /// Set the honorific suffixes, escaping and preserving the rest of the
    /// line.
    pub fn set_suffixes<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_component(4, values);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::{prop::n::N, tree::cst::VcardCst};

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
