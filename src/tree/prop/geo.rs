//! # GEO lens
//!
//! The `GEO` property lens. Its value shape is version-specific, so the lens
//! decodes through the card version: a `geo:` URI in 4.0
//! ([`VcardValue::Uri`]), a `lat;long` coordinate pair in 2.1 / 3.0
//! ([`VcardValue::Geo`]). The version-blind [`decode`](VcardPropLens::decode)
//! assumes the 4.0 URI form.

use crate::{
    prop::VCARD_GEO,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::{VcardValue, uri::VcardUri},
    version::VcardVersion,
};

/// The `GEO` property lens.
pub struct GEO;

impl VcardPropLens for GEO {
    const NAME: &'static str = VCARD_GEO;

    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardValue<'v> {
        VcardValue::Uri(VcardUri::decode(value))
    }

    fn decode_versioned<'v>(line: &'v VcardLine<'_>, version: &VcardVersion<'_>) -> VcardValue<'v> {
        line.decode_geo(version)
    }

    fn encode(decoded: &VcardValue<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
