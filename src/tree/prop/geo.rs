//! # GEO lens
//!
//! The `GEO` property lens: decoded as a URI (its 4.0 form, a `geo:` URI). In
//! 2.1 / 3.0 the value is instead a `lat;long` pair; that round-trips as a URI
//! string but is not semantically split here.

use crate::{
    prop::VCARD_GEO,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::uri::VcardUri,
};

/// The `GEO` property lens.
pub struct GEO;

impl VcardPropLens for GEO {
    const NAME: &'static str = VCARD_GEO;

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
