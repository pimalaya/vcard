//! # BDAY lens
//!
//! The `BDAY` (birthday) property lens: a date-and-or-time value.

use crate::{
    prop::VCARD_BDAY,
    tree::{cursor::VcardValueCursor, lens::VcardPropLens, line::VcardLine, value::VcardValueNode},
    value::datetime::VcardDateAndOrTime,
};

/// The `BDAY` property lens.
pub struct BDAY;

impl VcardPropLens for BDAY {
    const NAME: &'static str = VCARD_BDAY;

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
