//! # NAME lens
//!
//! Reading and editing the `NAME` property in place: it decodes as a
//! [`VcardText`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`NAME`].

use crate::{
    prop::name::NAME,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::text::VcardText,
};

impl VcardPropLens for NAME {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
