//! # SOUND lens
//!
//! The `SOUND` property lens: an audio resource, inline or by URI.

use crate::v2_1::{
    prop::VCARD_SOUND,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::binary::VcardBinary,
};

/// The `SOUND` property lens.
pub struct SOUND;

impl VcardPropLens for SOUND {
    const NAME: &'static str = VCARD_SOUND;

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
