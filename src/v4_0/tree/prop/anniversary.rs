//! # ANNIVERSARY lens
//!
//! The `ANNIVERSARY` property lens: a date-and-or-time value.

use crate::v4_0::{
    prop::VCARD_ANNIVERSARY,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::datetime::VcardDateAndOrTime,
};

/// The `ANNIVERSARY` property lens.
pub struct ANNIVERSARY;

impl VcardPropLens for ANNIVERSARY {
    const NAME: &'static str = VCARD_ANNIVERSARY;

    type Target<'v> = VcardDateAndOrTime<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardDateAndOrTime<'v> {
        VcardDateAndOrTime::decode(value)
    }

    fn encode(decoded: &VcardDateAndOrTime<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
