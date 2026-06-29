//! # PHOTO lens
//!
//! The `PHOTO` property lens: an image, inline or by URI.

use crate::v21::{
    prop::VCARD_PHOTO,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::binary::VcardBinary,
};

/// The `PHOTO` property lens.
pub struct PHOTO;

impl VcardPropLens for PHOTO {
    const NAME: &'static str = VCARD_PHOTO;

    type Target<'v> = VcardBinary<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>) -> VcardBinary<'v> {
        VcardBinary::decode(line)
    }

    fn encode(decoded: &VcardBinary<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
