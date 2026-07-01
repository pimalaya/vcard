//! # NICKNAME lens
//!
//! The `NICKNAME` property lens: the familiar or informal name(s) of the
//! object, decoded as a comma-separated `VcardTextList` (RFC 6350 6.2.3).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
    },
    value::{VcardValueKind, text::VcardTextList},
    version::VcardVersion,
};

/// The `NICKNAME` property lens.
pub struct NICKNAME;

impl VcardPropLens for NICKNAME {
    type Target<'v> = VcardTextList<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for NICKNAME {
    const KIND: VcardPropKind = VcardPropKind::Nickname;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0, VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::TextList]
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
