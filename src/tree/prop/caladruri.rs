//! # CALADRURI lens
//!
//! Reading and editing the `CALADRURI` property in place: it decodes as a
//! [`VcardUri`] and edits through the generic [`VcardValueCursor`].
//!
//! Its RFC contract sits on the marker, [`CALADRURI`].

use crate::{
    prop::caladruri::CALADRURI,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::uri::VcardUri,
};

impl VcardPropLens for CALADRURI {
    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
