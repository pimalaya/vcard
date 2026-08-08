//! # LANG lens
//!
//! The `LANG` property lens: a language the contact may be contacted in,
//! decoded as an RFC 5646 language tag.
//!
//! See RFC 6350 6.4.4.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{lens::VcardPropLens, spec::VcardPropSpec},
        value::cursor::VcardValueCursor,
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

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for LANG {
    const KIND: VcardPropKind = VcardPropKind::Lang;

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
