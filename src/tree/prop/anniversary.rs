//! # ANNIVERSARY lens
//!
//! The `ANNIVERSARY` property lens: a date-and-or-time value.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropCardinality, VcardPropLens, VcardPropSpec},
        value::VcardValueNode,
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

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardDateAndOrTime<'v> {
        VcardDateAndOrTime::decode(value)
    }

    fn encode(decoded: &VcardDateAndOrTime<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for ANNIVERSARY {
    const PROP: VcardPropKind = VcardPropKind::Anniversary;

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
