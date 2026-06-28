//! # FN lens
//!
//! The `FN` (formatted name) property lens: a single text value.

use crate::{
    prop::VCARD_FN,
    tree::{cursor::VcardValueCursor, lens::VcardPropLens, line::VcardLine, value::VcardValueNode},
    value::text::VcardText,
};

/// The `FN` property lens.
pub struct FN;

impl VcardPropLens for FN {
    const NAME: &'static str = VCARD_FN;

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
