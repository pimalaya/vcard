//! # LANG lens
//!
//! The `LANG` property lens: an RFC 5646 language tag.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
        value::VcardValueNode,
    },
    value::{VcardValueKind, language::VcardLanguageTag},
    version::VcardVersion,
};

/// The `LANG` property lens.
pub struct LANG;

impl VcardPropLens for LANG {
    type Target<'v> = VcardLanguageTag<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardLanguageTag<'v> {
        VcardLanguageTag::decode(value)
    }

    fn encode(decoded: &VcardLanguageTag<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for LANG {
    const PROP: VcardPropKind = VcardPropKind::Lang;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::LanguageTag]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
