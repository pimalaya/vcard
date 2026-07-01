//! # FBURL lens
//!
//! The `FBURL` property lens: the URL to the contact's free/busy information,
//! decoded as a URI.
//!
//! See RFC 6350 6.9.1.

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

/// The `FBURL` property lens.
pub struct FBURL;

impl VcardPropLens for FBURL {
    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for FBURL {
    const KIND: VcardPropKind = VcardPropKind::FbUrl;

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
