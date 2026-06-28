//! # REV lens
//!
//! The `REV` (revision) property lens: a timestamp value.

use crate::{
    prop::VCARD_REV,
    tree::{cursor::VcardValueCursor, lens::VcardPropLens, line::VcardLine, value::VcardValueNode},
    value::datetime::VcardTimestamp,
};

/// The `REV` property lens.
pub struct REV;

impl VcardPropLens for REV {
    const NAME: &'static str = VCARD_REV;

    type Target<'v> = VcardTimestamp<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardTimestamp<'v> {
        VcardTimestamp::decode(value)
    }

    fn encode(decoded: &VcardTimestamp<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
