//! # N lens
//!
//! The `N` property lens and its bespoke edit cursor.
//!
//! `N` is the showcase of the structured-value path: unlike the scalar lenses
//! that use the generic cursor, it pairs [`VcardN`] with a dedicated [`NCursor`]
//! that names the five single-valued components, so callers write
//! `cursor.set_family(...)` rather than `cursor.set_component(0, ...)`. The
//! decode/encode projections still delegate to [`VcardN`]'s inherent methods (in
//! [`crate::v21::tree::decode`] / [`crate::v21::tree::encode`]); only the cursor
//! is bespoke. Edits are byte preserving: writing one component leaves the
//! others, and every parameter, untouched.

use alloc::borrow::Cow;

use crate::v21::{
    prop::VCARD_N,
    tree::{line::VcardLine, param::VcardParamLens, prop::VcardPropLens, value::VcardValueNode},
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

    fn decode<'v>(line: &'v VcardLine<'_>) -> VcardN<'v> {
        VcardN::decode(line)
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
        VcardN::decode(self.line)
    }

    /// The family name, decoded.
    pub fn family(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().next().unwrap_or_default()
    }

    /// The given name, decoded.
    pub fn given(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(1).unwrap_or_default()
    }

    /// The additional name, decoded.
    pub fn additional(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(2).unwrap_or_default()
    }

    /// The honorific prefix, decoded.
    pub fn prefix(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(3).unwrap_or_default()
    }

    /// The honorific suffix, decoded.
    pub fn suffix(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(4).unwrap_or_default()
    }

    /// Set the family name, escaping and preserving the rest of the line.
    pub fn set_family(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(0, value.as_ref());
    }

    /// Set the given name, escaping and preserving the rest of the line.
    pub fn set_given(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(1, value.as_ref());
    }

    /// Set the additional name, escaping and preserving the rest of the line.
    pub fn set_additional(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(2, value.as_ref());
    }

    /// Set the honorific prefix, escaping and preserving the rest of the line.
    pub fn set_prefix(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(3, value.as_ref());
    }

    /// Set the honorific suffix, escaping and preserving the rest of the line.
    pub fn set_suffix(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(4, value.as_ref());
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::v21::tree::{cst::VcardCst, prop::n::N};

    #[test]
    fn names_components_and_sets_them_preserving_the_rest() {
        let mut card =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nN:Doe;John;;;\r\nEND:VCARD\r\n")
                .unwrap();

        assert_eq!(card.prop_mut::<N>().unwrap().family(), "Doe");

        card.prop_mut::<N>().unwrap().set_given("Jane");
        assert!(card.to_string().contains("N:Doe;Jane;;;\r\n"));
    }
}
