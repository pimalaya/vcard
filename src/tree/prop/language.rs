//! # LANGUAGE lens
//!
//! Reading and editing the `LANGUAGE` property in place: it decodes as a
//! [`VcardLanguageTag`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`LANGUAGE`].

use crate::{
    prop::language::LANGUAGE,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::language::VcardLanguageTag,
};

impl VcardPropLens for LANGUAGE {
    type Target<'v> = VcardLanguageTag<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
