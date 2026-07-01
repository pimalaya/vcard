//! # ORG lens
//!
//! The `ORG` (organization) property lens (RFC 6350 6.6.4): the organizational
//! name and units the object belongs to. Its value is a `;`-ordered list of
//! units, which maps cleanly onto the generic [`VcardValueCursor`]'s positional
//! component access, so it needs no bespoke cursor.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
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

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for ORG {
    const KIND: VcardPropKind = VcardPropKind::Org;

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
