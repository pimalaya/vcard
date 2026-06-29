//! # LANG lens
//!
//! The `LANG` property lens: an RFC 5646 language tag.

use crate::{
    prop::VCARD_LANG,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::language::VcardLanguageTag,
};

/// The `LANG` property lens.
pub struct LANG;

impl VcardPropLens for LANG {
    const NAME: &'static str = VCARD_LANG;

    type Target<'v> = VcardLanguageTag<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardLanguageTag<'v> {
        VcardLanguageTag::decode(value)
    }

    fn encode(decoded: &VcardLanguageTag<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
