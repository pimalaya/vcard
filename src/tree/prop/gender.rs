//! # GENDER lens
//!
//! The `GENDER` property lens, pairing a sex code with a free-text identity,
//! with a cursor naming those two components.
//!
//! See RFC 6350 6.2.7.

use alloc::borrow::Cow;

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        codec::VcardCodec,
        line::VcardLine,
        param::lens::VcardParamLens,
        prop::{cardinality::VcardPropCardinality, lens::VcardPropLens, spec::VcardPropSpec},
    },
    value::{VcardValueKind, gender::VcardGender},
    version::VcardVersion,
};

/// The `GENDER` property lens.
pub struct GENDER;

impl VcardPropLens for GENDER {
    type Target<'v> = VcardGender<'v>;

    type Cursor<'c, 'a>
        = VcardGenderCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardGenderCursor<'c, 'a> {
        VcardGenderCursor { line }
    }
}

impl VcardPropSpec for GENDER {
    const KIND: VcardPropKind = VcardPropKind::Gender;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Gender]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}

/// A typed cursor over a GENDER line, naming its sex code and identity.
pub struct VcardGenderCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl VcardGenderCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardGender<'_> {
        VcardGender::decode(&self.line.value)
    }

    /// The sex code, decoded.
    pub fn sex(&self) -> Cow<'_, str> {
        self.line.value.decode_component(0)
    }

    /// Set the sex code.
    pub fn set_sex(&mut self, value: impl AsRef<str>) {
        self.line.value.set_component(0, &[value]);
    }

    /// The free-text gender identity, decoded.
    pub fn identity(&self) -> Cow<'_, str> {
        self.line.value.decode_component(1)
    }

    /// Set the gender identity.
    pub fn set_identity(&mut self, value: impl AsRef<str>) {
        self.line.value.set_component(1, &[value]);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}
