//! # REV lens
//!
//! The `REV` (revision) property lens: a timestamp value.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropCardinality, VcardPropLens, VcardPropSpec},
        value::VcardValueNode,
    },
    value::{VcardValueKind, datetime::VcardTimestamp},
    version::VcardVersion,
};

/// The `REV` property lens.
pub struct REV;

impl VcardPropLens for REV {
    type Target<'v> = VcardTimestamp<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardTimestamp<'v> {
        VcardTimestamp::decode(value)
    }

    fn encode(decoded: &VcardTimestamp<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for REV {
    const PROP: VcardPropKind = VcardPropKind::Rev;

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Timestamp]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}
