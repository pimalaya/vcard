//! # N lens
//!
//! The `N` (structured name) property lens and its bespoke edit cursor: the
//! components of the name of the object the card represents (RFC 6350 6.2.2).
//!
//! `N` is the showcase of the structured-value path: unlike the scalar/list
//! lenses that use the generic cursor, it pairs [`VcardN`] with a dedicated
//! [`VcardNCursor`] that names the five components, so callers write
//! `cursor.set_family(...)` rather than `cursor.set_component(0, ...)`. The
//! decode/encode projections use the lens defaults, which delegate to
//! [`VcardN`]'s [`Codec`] impl in [`crate::tree::value`]; only the cursor is
//! bespoke. Edits are byte preserving: writing one component leaves the others,
//! and every parameter, untouched.

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        codec::Codec,
        line::VcardLine,
        param::VcardParamLens,
        prop::{VcardPropCardinality, VcardPropLens, VcardPropSpec},
    },
    value::{VcardValueKind, n::VcardN},
    version::VcardVersion,
};

/// The `N` property lens.
pub struct N;

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

impl VcardPropSpec for N {
    const KIND: VcardPropKind = VcardPropKind::N;

    fn cardinality(version: VcardVersion) -> VcardPropCardinality {
        match version {
            VcardVersion::V4_0 => VcardPropCardinality::AtMostOne,
            _ => VcardPropCardinality::ExactlyOne,
        }
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::N]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::SortAs,
            VcardParamKind::Language,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
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

    /// Set the honorific prefixes, escaping and preserving the rest of the
    /// line.
    pub fn set_prefixes<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(3, values);
    }

    /// Set the honorific suffixes, escaping and preserving the rest of the
    /// line.
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
