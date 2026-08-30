//! # MAILER lens
//!
//! Reading and editing the `MAILER` property in place: it decodes as a
//! [`VcardText`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`MAILER`].

use crate::{
    prop::mailer::MAILER,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::text::VcardText,
};

impl VcardPropLens for MAILER {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
