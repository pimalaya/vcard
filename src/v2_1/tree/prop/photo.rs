//! # PHOTO lens
//!
//! The `PHOTO` property lens: a URI to an image (a `data:` URI in 4.0).

use crate::v2_1::{
    prop::VCARD_PHOTO,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::uri::VcardUri,
};

/// The `PHOTO` property lens.
pub struct PHOTO;

impl VcardPropLens for PHOTO {
    const NAME: &'static str = VCARD_PHOTO;

    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardUri<'v> {
        VcardUri::decode(value)
    }

    fn encode(decoded: &VcardUri<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
