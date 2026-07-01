//! # FN lens
//!
//! The `FN` (formatted name) property lens: the contact's display name, decoded
//! as a single text value.
//!
//! See RFC 6350 6.2.1.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropCardinality, VcardPropLens, VcardPropSpec},
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `FN` property lens.
pub struct FN;

impl VcardPropLens for FN {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for FN {
    const KIND: VcardPropKind = VcardPropKind::Fn;

    fn cardinality(version: VcardVersion) -> VcardPropCardinality {
        match version {
            VcardVersion::V2_1 => VcardPropCardinality::AtMostOne,
            _ => VcardPropCardinality::OneOrMore,
        }
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Language,
            VcardParamKind::AltId,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Value,
        ]
    }
}
