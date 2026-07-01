//! # CALURI lens
//!
//! The `CALURI` property lens: the URI of the contact's calendar, decoded as a
//! URI.
//!
//! See RFC 6350 6.9.3.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
        value::VcardValueCursor,
    },
    value::{VcardValueKind, uri::VcardUri},
    version::VcardVersion,
};

/// The `CALURI` property lens.
pub struct CALURI;

impl VcardPropLens for CALURI {
    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for CALURI {
    const KIND: VcardPropKind = VcardPropKind::CalUri;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Uri]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
