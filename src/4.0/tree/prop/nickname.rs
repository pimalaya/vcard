//! # NICKNAME lens
//!
//! The `NICKNAME` property lens: a comma-separated text list.

use crate::v40::{
    prop::VCARD_NICKNAME,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardTextList,
};

/// The `NICKNAME` property lens.
pub struct NICKNAME;

impl VcardPropLens for NICKNAME {
    const NAME: &'static str = VCARD_NICKNAME;

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
