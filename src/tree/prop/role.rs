//! # ROLE lens
//!
//! Reading and editing the `ROLE` property in place: it decodes as a
//! [`VcardText`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`ROLE`].

use crate::{
    prop::role::ROLE,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::text::VcardText,
};

impl VcardPropLens for ROLE {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
