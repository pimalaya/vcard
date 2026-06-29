//! # LABEL lens
//!
//! The `LABEL` property lens: a formatted address label, as a single text value.

use crate::v21::{
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
