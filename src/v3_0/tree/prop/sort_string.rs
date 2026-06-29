//! # SORT-STRING lens
//!
//! The `SORT-STRING` property lens: the text to sort the card by, a single text
//! value.

use crate::v3_0::{
    prop::VCARD_SORT_STRING,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `SORT-STRING` property lens.
#[allow(non_camel_case_types)]
pub struct SORT_STRING;

impl VcardPropLens for SORT_STRING {
    const NAME: &'static str = VCARD_SORT_STRING;

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
