//! # TZ lens
//!
//! The `TZ` (time zone) property lens: decoded as a text value (its default
//! form; the UTC-offset and URI forms round-trip as text).

use crate::{
    prop::VCARD_TZ,
    tree::{cursor::VcardValueCursor, lens::VcardPropLens, line::VcardLine, value::VcardValueNode},
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
