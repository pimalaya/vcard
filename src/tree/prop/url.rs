//! # URL lens
//!
//! The `URL` property lens: a URL pointing to a resource representing the
//! object, decoded as a `VcardUri` (RFC 6350 6.7.8).

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

/// The `URL` property lens.
pub struct URL;

impl VcardPropLens for URL {
    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for URL {
    const KIND: VcardPropKind = VcardPropKind::Url;

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
