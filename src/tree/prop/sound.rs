//! # SOUND lens
//!
//! Reading and editing the `SOUND` property in place.
//!
//! Its value shape is version-specific, so the lens overrides
//! [`decode`](VcardPropLens::decode) to resolve it from the card version
//! through the property spec: a `data:` URI in 4.0 ([`VcardValue::Uri`]),
//! inline base64 or a URI reference in 2.1 / 3.0 ([`VcardValue::Binary`]).
//!
//! Its RFC contract sits on the marker, [`SOUND`].

use crate::{
    prop::VcardPropKind,
    prop::sound::SOUND,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::VcardValue,
    version::VcardVersion,
};

impl VcardPropLens for SOUND {
    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>, version: VcardVersion) -> VcardValue<'v> {
        line.decode_value(VcardPropKind::Sound, version)
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
