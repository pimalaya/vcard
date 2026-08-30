//! # REV lens
//!
//! Reading and editing the `REV` property in place: it decodes as a
//! [`VcardTimestamp`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`REV`].

use crate::{
    prop::rev::REV,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::datetime::VcardTimestamp,
};

impl VcardPropLens for REV {
    type Target<'v> = VcardTimestamp<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
