//! # CATEGORIES lens
//!
//! Reading and editing the `CATEGORIES` property in place: it decodes as a
//! [`VcardTextList`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`CATEGORIES`].

use crate::{
    prop::categories::CATEGORIES,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::text::VcardTextList,
};

impl VcardPropLens for CATEGORIES {
    type Target<'v> = VcardTextList<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
