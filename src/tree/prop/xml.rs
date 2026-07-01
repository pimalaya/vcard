//! # XML lens
//!
//! The `XML` property lens: extended XML-encoded vCard data that fits no other
//! property, decoded as a single `VcardText` (RFC 6350 6.1.5).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `XML` property lens.
pub struct XML;

impl VcardPropLens for XML {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for XML {
    const KIND: VcardPropKind = VcardPropKind::Xml;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::AltId, VcardParamKind::Value]
    }
}
