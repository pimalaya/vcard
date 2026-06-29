//! # TZ lens
//!
//! The `TZ` (time zone) property lens: decoded as a UTC offset (its default form
//! in vCard 3.0; the text form round-trips through the same kind).

use crate::v3_0::{
    prop::VCARD_TZ,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::utc_offset::VcardUtcOffset,
};

/// The `TZ` property lens.
pub struct TZ;

impl VcardPropLens for TZ {
    const NAME: &'static str = VCARD_TZ;

    type Target<'v> = VcardUtcOffset<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardUtcOffset<'v> {
        VcardUtcOffset::decode(value)
    }

    fn encode(decoded: &VcardUtcOffset<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
