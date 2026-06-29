//! # SOUND lens
//!
//! The `SOUND` property lens: a URI to an audio resource.

use crate::v40::{
    prop::VCARD_SOUND,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::uri::VcardUri,
};

/// The `SOUND` property lens.
pub struct SOUND;

impl VcardPropLens for SOUND {
    const NAME: &'static str = VCARD_SOUND;

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
