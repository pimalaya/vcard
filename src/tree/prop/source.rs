//! # SOURCE lens
//!
//! The `SOURCE` property lens: a URI from which the directory information of
//! the card can be retrieved, decoded as a `VcardUri` (RFC 6350 6.1.3).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{lens::VcardPropLens, spec::VcardPropSpec},
        value::cursor::VcardValueCursor,
    },
    value::{VcardValueKind, uri::VcardUri},
    version::VcardVersion,
};

/// The `SOURCE` property lens.
pub struct SOURCE;

impl VcardPropLens for SOURCE {
    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for SOURCE {
    const KIND: VcardPropKind = VcardPropKind::Source;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0, VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Uri]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::AltId,
            VcardParamKind::MediaType,
            VcardParamKind::Value,
        ]
    }
}
