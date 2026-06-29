//! # CATEGORIES lens
//!
//! The `CATEGORIES` property lens: a comma-separated text list.

use crate::v2_1::{
    prop::VCARD_CATEGORIES,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardTextList,
};

/// The `CATEGORIES` property lens.
pub struct CATEGORIES;

impl VcardPropLens for CATEGORIES {
    const NAME: &'static str = VCARD_CATEGORIES;

    type Target<'v> = VcardTextList<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardTextList<'v> {
        VcardTextList::decode(value)
    }

    fn encode(decoded: &VcardTextList<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
