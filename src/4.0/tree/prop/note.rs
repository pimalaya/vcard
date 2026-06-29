//! # NOTE lens
//!
//! The `NOTE` property lens: a free-text note, as a single text value.

use crate::v40::{
    prop::VCARD_NOTE,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `NOTE` property lens.
pub struct NOTE;

impl VcardPropLens for NOTE {
    const NAME: &'static str = VCARD_NOTE;

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
