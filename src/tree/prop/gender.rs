//! # GENDER lens
//!
//! The `GENDER` property lens, with a cursor naming its two components (a sex
//! code and a free-text identity).

use alloc::borrow::Cow;

use crate::{
    prop::VCARD_GENDER,
    tree::{
        lens::{VcardParamLens, VcardPropLens},
        line::VcardLine,
        value::VcardValueNode,
    },
    value::gender::VcardGender,
};

/// The `GENDER` property lens.
pub struct GENDER;

impl VcardPropLens for GENDER {
    const NAME: &'static str = VCARD_GENDER;

    type Target<'v> = VcardGender<'v>;

    type Cursor<'c, 'a>
        = GenderCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardGender<'v> {
        VcardGender::decode(value)
    }

    fn encode(decoded: &VcardGender<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> GenderCursor<'c, 'a> {
        GenderCursor { line }
    }
}

/// A typed cursor over a GENDER line, naming its sex code and identity.
pub struct GenderCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl GenderCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardGender<'_> {
        VcardGender::decode(&self.line.value)
    }

    /// The sex code, decoded.
    pub fn sex(&self) -> Cow<'_, str> {
        self.line.value.decode_scalar_at(0)
    }

    /// Set the sex code.
    pub fn set_sex(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(0, &[value]);
    }

    /// The free-text gender identity, decoded.
    pub fn identity(&self) -> Cow<'_, str> {
        self.line.value.decode_scalar_at(1)
    }

    /// Set the gender identity.
    pub fn set_identity(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(1, &[value]);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}
