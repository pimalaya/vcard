//! # SORTSTRING lens
//!
//! The `SORTSTRING` property lens: the sort string (vCard 3.0), as a single text value.

use crate::{
    prop::VCARD_SORT_STRING,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `SORTSTRING` property lens.
pub struct SORTSTRING;

impl VcardPropLens for SORTSTRING {
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
