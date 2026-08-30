//! # ORG lens
//!
//! Reading and editing the `ORG` property in place: it decodes as a
//! [`VcardOrg`], and its `;`-ordered units map cleanly onto the generic
//! [`VcardValueCursor`]'s positional component access, so it needs no bespoke
//! cursor.
//!
//! Its RFC contract sits on the marker, [`ORG`].

use crate::{
    prop::org::ORG,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::org::VcardOrg,
};

impl VcardPropLens for ORG {
    type Target<'v> = VcardOrg<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
