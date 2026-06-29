//! # SOUND lens
//!
//! The `SOUND` property lens. Its value shape is version-specific, so the lens
//! decodes through the card version: a `data:` URI in 4.0
//! ([`VcardValue::Uri`]), inline base64 or a URI reference in 2.1 / 3.0
//! ([`VcardValue::Binary`]). The version-blind
//! [`decode`](VcardPropLens::decode) assumes the 4.0 URI form.

use crate::{
    prop::VCARD_SOUND,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::{VcardValue, uri::VcardUri},
    version::VcardVersion,
};

/// The `SOUND` property lens.
pub struct SOUND;

impl VcardPropLens for SOUND {
    const NAME: &'static str = VCARD_SOUND;

    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardValue<'v> {
        VcardValue::Uri(VcardUri::decode(value))
    }

    fn decode_versioned<'v>(line: &'v VcardLine<'_>, version: &VcardVersion<'_>) -> VcardValue<'v> {
        line.decode_binary_value(version)
    }

    fn encode(decoded: &VcardValue<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
