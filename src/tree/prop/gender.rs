//! # GENDER lens
//!
//! Reading and editing the `GENDER` property in place through a cursor naming
//! its two components, the sex code and the free-text identity.
//!
//! Its RFC contract sits on the marker, [`GENDER`].

use alloc::borrow::Cow;

use crate::{
    prop::gender::GENDER,
    tree::{
        codec::VcardCodec, line::VcardLine, param::lens::VcardParamLens, prop::lens::VcardPropLens,
    },
    value::gender::VcardGender,
};

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
