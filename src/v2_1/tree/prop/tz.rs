//! # TZ lens
//!
//! The `TZ` (time zone) property lens: a single text value.

use crate::v2_1::{
    prop::VCARD_TZ,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `TZ` property lens.
pub struct TZ;

impl VcardPropLens for TZ {
    const NAME: &'static str = VCARD_TZ;

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
