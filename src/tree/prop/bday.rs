//! # BDAY lens
//!
//! The `BDAY` (birthday) property lens: the contact's birth date, decoded as a
//! date-and-or-time value.
//!
//! See RFC 6350 6.2.5.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{VcardPropCardinality, VcardPropLens, VcardPropSpec},
        value::VcardValueCursor,
    },
    value::{VcardValueKind, datetime::VcardDateAndOrTime},
    version::VcardVersion,
};

/// The `BDAY` property lens.
pub struct BDAY;

impl VcardPropLens for BDAY {
    type Target<'v> = VcardDateAndOrTime<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for BDAY {
    const KIND: VcardPropKind = VcardPropKind::Bday;

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
            VcardParamKind::Language,
            VcardParamKind::Value,
        ]
    }
}
