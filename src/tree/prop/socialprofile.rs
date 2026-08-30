//! # SOCIALPROFILE lens
//!
//! Reading and editing the `SOCIALPROFILE` property in place.
//!
//! Its value takes two kinds, so the lens overrides
//! [`decode`](VcardPropLens::decode) to resolve the one in force through the
//! property spec: a URI by default, a username when `VALUE=text` is declared.
//!
//! Its RFC contract sits on the marker, [`SOCIALPROFILE`].

use crate::{
    prop::VcardPropKind,
    prop::socialprofile::SOCIALPROFILE,
    tree::{line::VcardLine, prop::lens::VcardPropLens, value::cursor::VcardValueCursor},
    value::VcardValue,
    version::VcardVersion,
};

impl VcardPropLens for SOCIALPROFILE {
    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>, version: VcardVersion) -> VcardValue<'v> {
        line.decode_value(VcardPropKind::SocialProfile, version)
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
