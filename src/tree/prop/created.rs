//! # CREATED lens
//!
//! Reading and editing the `CREATED` property in place: it decodes as a
//! [`VcardTimestamp`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`CREATED`].

use crate::{
    prop::created::CREATED,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::datetime::VcardTimestamp,
};

impl VcardPropLens for CREATED {
    type Target<'v> = VcardTimestamp<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
