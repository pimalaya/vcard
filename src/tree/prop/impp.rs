//! # IMPP lens
//!
//! The `IMPP` property lens: a URI for instant messaging and presence with the
//! contact, decoded as a URI.
//!
//! See RFC 6350 6.4.3.

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

/// The `IMPP` property lens.
pub struct IMPP;

impl VcardPropLens for IMPP {
    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for IMPP {
    const KIND: VcardPropKind = VcardPropKind::Impp;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0, VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Uri]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::MediaType,
            VcardParamKind::Value,
        ]
    }
}
