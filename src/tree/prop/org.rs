//! # ORG lens
//!
//! The `ORG` (organization) property lens. Its value is a `;`-ordered list of
//! units, which maps cleanly onto the generic [`VcardValueCursor`]'s positional
//! component access, so it needs no bespoke cursor.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
        value::VcardValueNode,
    },
    value::{VcardValueKind, org::VcardOrg},
    version::VcardVersion,
};

/// The `ORG` property lens.
pub struct ORG;

impl VcardPropLens for ORG {
    type Target<'v> = VcardOrg<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardOrg<'v> {
        VcardOrg::decode(value)
    }

    fn encode(decoded: &VcardOrg<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for ORG {
    const PROP: VcardPropKind = VcardPropKind::Org;

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Org]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::SortAs,
            VcardParamKind::Language,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::Type,
            VcardParamKind::Value,
        ]
    }
}
