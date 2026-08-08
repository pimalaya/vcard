//! # ANNIVERSARY lens
//!
//! The `ANNIVERSARY` property lens: the contact's anniversary, decoded as a
//! date-and-or-time value.
//!
//! See RFC 6350 6.2.6.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{cardinality::VcardPropCardinality, lens::VcardPropLens, spec::VcardPropSpec},
        value::cursor::VcardValueCursor,
    },
    value::{VcardValueKind, datetime::VcardDateAndOrTime},
    version::VcardVersion,
};

/// The `ANNIVERSARY` property lens.
pub struct ANNIVERSARY;

impl VcardPropLens for ANNIVERSARY {
    type Target<'v> = VcardDateAndOrTime<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for ANNIVERSARY {
    const KIND: VcardPropKind = VcardPropKind::Anniversary;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::DateAndOrTime, VcardValueKind::Text]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::AltId,
            VcardParamKind::CalScale,
            VcardParamKind::Value,
        ]
    }
}
