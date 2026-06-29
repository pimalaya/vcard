//! # LOGO lens
//!
//! The `LOGO` property lens: a logo image, inline (`ENCODING=b`) or by URI
//! (`VALUE=uri`), decoded as a [`VcardBinary`].

use crate::v3_0::{
    prop::VCARD_LOGO,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::binary::VcardBinary,
};

/// The `LOGO` property lens.
pub struct LOGO;

impl VcardPropLens for LOGO {
    const NAME: &'static str = VCARD_LOGO;

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
