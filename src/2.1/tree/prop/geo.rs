//! # GEO lens
//!
//! The `GEO` (geographic position) property lens. Its value is a `lat;long`
//! pair, two single-valued components that map cleanly onto the generic
//! [`VcardValueCursor`]'s positional component access, so it needs no bespoke
//! cursor.

use crate::v21::{
    prop::VCARD_GEO,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::geo::VcardGeo,
};

/// The `GEO` property lens.
pub struct GEO;

impl VcardPropLens for GEO {
    const NAME: &'static str = VCARD_GEO;

    type Target<'v> = VcardGeo<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>) -> VcardGeo<'v> {
        VcardGeo::decode(line)
    }

    fn encode(decoded: &VcardGeo<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
