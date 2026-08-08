//! # PRONOUNS lens
//!
//! The `PRONOUNS` property lens: pronouns to refer to the contact by,
//! decoded as text.
//!
//! See RFC 9554.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{lens::VcardPropLens, spec::VcardPropSpec},
        value::cursor::VcardValueCursor,
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `PRONOUNS` property lens.
pub struct PRONOUNS;

impl VcardPropLens for PRONOUNS {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for PRONOUNS {
    const KIND: VcardPropKind = VcardPropKind::Pronouns;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
