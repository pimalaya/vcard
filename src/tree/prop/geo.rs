//! # GEO lens
//!
//! Reading and editing the `GEO` property in place.
//!
//! Its value shape is version-specific, so the lens overrides
//! [`decode`](VcardPropLens::decode) to resolve it from the card version
//! through the property spec: a `geo:` URI in 4.0 ([`VcardValue::Uri`]), a
//! `lat;long` coordinate pair in 2.1 / 3.0 ([`VcardValue::Geo`]).
//!
//! Its RFC contract sits on the marker, [`GEO`].

use crate::{
    prop::VcardPropKind,
    prop::geo::GEO,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::VcardValue,
    version::VcardVersion,
};

impl VcardPropLens for GEO {
    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>, version: VcardVersion) -> VcardValue<'v> {
        line.decode_value(VcardPropKind::Geo, version)
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
