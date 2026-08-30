//! # ANNIVERSARY lens
//!
//! Reading and editing the `ANNIVERSARY` property in place: it decodes as a
//! [`VcardDateAndOrTime`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`ANNIVERSARY`].

use crate::{
    prop::anniversary::ANNIVERSARY,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::datetime::VcardDateAndOrTime,
};

impl VcardPropLens for ANNIVERSARY {
    type Target<'v> = VcardDateAndOrTime<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
