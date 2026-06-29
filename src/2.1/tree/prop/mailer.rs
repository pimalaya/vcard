//! # MAILER lens
//!
//! The `MAILER` property lens: the email client used, as a single text value.

use crate::v21::{
    prop::VCARD_MAILER,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::text::VcardText,
};

/// The `MAILER` property lens.
pub struct MAILER;

impl VcardPropLens for MAILER {
    const NAME: &'static str = VCARD_MAILER;

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
