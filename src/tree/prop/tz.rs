//! # TZ lens
//!
//! The `TZ` (time zone) property lens: the time zone of the object. The lens
//! decodes it as a single `VcardText` (its default form; the UTC-offset and URI
//! forms round-trip as text) (RFC 6350 6.5.1).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `TZ` property lens.
pub struct TZ;

impl VcardPropLens for TZ {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for TZ {
    const KIND: VcardPropKind = VcardPropKind::Tz;

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::AltId,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::Value,
        ]
    }
}
