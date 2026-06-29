//! # UID lens
//!
//! The `UID` property lens: a unique identifier, as a single text value.

use crate::v2_1::{
    prop::VCARD_UID,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `UID` property lens.
pub struct UID;

impl VcardPropLens for UID {
    const NAME: &'static str = VCARD_UID;

    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>) -> VcardText<'v> {
        VcardText::decode(line)
    }

    fn encode(decoded: &VcardText<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
