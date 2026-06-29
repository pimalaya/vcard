//! # SOUND lens
//!
//! The `SOUND` property lens: an audio resource, inline (`ENCODING=b`) or by URI
//! (`VALUE=uri`), decoded as a [`VcardBinary`].

use crate::v30::{
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

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardBinary<'v> {
        VcardBinary::decode(value)
    }

    fn encode(decoded: &VcardBinary<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
