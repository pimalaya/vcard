//! # URL lens
//!
//! Reading and editing the `URL` property in place: it decodes as a
//! [`VcardUri`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`URL`].

use crate::{
    prop::url::URL,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::uri::VcardUri,
};

impl VcardPropLens for URL {
    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
