//! # KIND lens
//!
//! The `KIND` property lens: the kind of entity, as a text value.

use crate::{
    prop::VCARD_KIND,
    tree::{cursor::VcardValueCursor, lens::VcardPropLens, line::VcardLine, value::VcardValueNode},
    value::text::VcardText,
};

/// The `KIND` property lens.
pub struct KIND;

impl VcardPropLens for KIND {
    const NAME: &'static str = VCARD_KIND;

    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardText<'v> {
        VcardText::decode(value)
    }

    fn encode(decoded: &VcardText<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
