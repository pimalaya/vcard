//! # LABEL lens
//!
//! The `LABEL` property lens: the formatted text of a delivery address, a single
//! text value.

use crate::v3_0::{
    prop::VCARD_LABEL,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `LABEL` property lens.
pub struct LABEL;

impl VcardPropLens for LABEL {
    const NAME: &'static str = VCARD_LABEL;

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
