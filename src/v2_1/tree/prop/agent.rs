//! # AGENT lens
//!
//! The `AGENT` property lens: an agent for the entity, as a single text value.

use crate::v2_1::{
    prop::VCARD_AGENT,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `AGENT` property lens.
pub struct AGENT;

impl VcardPropLens for AGENT {
    const NAME: &'static str = VCARD_AGENT;

    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>) -> VcardText<'v> {
        VcardText::decode(line)
    }

    fn encode(decoded: &VcardText<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
