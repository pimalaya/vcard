//! # KEY lens
//!
//! The `KEY` property lens: a public key or certificate, inline (`ENCODING=b`)
//! or by URI (`VALUE=uri`), decoded as a [`VcardBinary`].

use crate::v30::{
    prop::VCARD_KEY,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::binary::VcardBinary,
};

/// The `KEY` property lens.
pub struct KEY;

impl VcardPropLens for KEY {
    const NAME: &'static str = VCARD_KEY;

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
