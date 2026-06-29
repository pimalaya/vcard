//! # LOGO lens
//!
//! The `LOGO` property lens: a URI to a logo image.

use crate::{
    prop::VCARD_LOGO,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::uri::VcardUri,
};

/// The `LOGO` property lens.
pub struct LOGO;

impl VcardPropLens for LOGO {
    const NAME: &'static str = VCARD_LOGO;

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
