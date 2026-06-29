//! # PROFILE lens
//!
//! The `PROFILE` property lens: the profile declaration (vCard 3.0), as a single text value.

use crate::{
    prop::VCARD_PROFILE,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `PROFILE` property lens.
pub struct PROFILE;

impl VcardPropLens for PROFILE {
    const NAME: &'static str = VCARD_PROFILE;

    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardText<'v> {
        VcardText::decode(value)
    }

    fn encode(decoded: &VcardText<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
