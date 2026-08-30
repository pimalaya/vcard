//! # NICKNAME lens
//!
//! Reading and editing the `NICKNAME` property in place: it decodes as a
//! [`VcardTextList`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`NICKNAME`].

use crate::{
    prop::nickname::NICKNAME,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::text::VcardTextList,
};

impl VcardPropLens for NICKNAME {
    type Target<'v> = VcardTextList<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
