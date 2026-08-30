//! # LANG lens
//!
//! Reading and editing the `LANG` property in place: it decodes as a
//! [`VcardLanguageTag`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`LANG`].

use crate::{
    prop::lang::LANG,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::language::VcardLanguageTag,
};

impl VcardPropLens for LANG {
    type Target<'v> = VcardLanguageTag<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
