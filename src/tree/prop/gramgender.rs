//! # GRAMGENDER lens
//!
//! The `GRAMGENDER` property lens: the grammatical gender to address the
//! contact by (e.g. `feminine`, `neuter`), decoded as text.
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

/// The `GRAMGENDER` property lens.
pub struct GRAMGENDER;

impl VcardPropLens for GRAMGENDER {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for GRAMGENDER {
    const KIND: VcardPropKind = VcardPropKind::GramGender;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
