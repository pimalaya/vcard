//! # GEO lens
//!
//! The `GEO` property lens. Its value shape is version-specific, so the lens
//! decodes through the card version: a `geo:` URI in 4.0
//! ([`VcardValue::Uri`]), a `lat;long` coordinate pair in 2.1 / 3.0
//! ([`VcardValue::Geo`]). The lens overrides
//! [`decode`](VcardPropLens::decode) to resolve the shape from the version
//! through the property spec.
//!
//! See RFC 6350 6.5.2.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
    },
    value::{VcardValue, VcardValueKind},
    version::VcardVersion,
};

/// The `GEO` property lens.
pub struct GEO;

impl VcardPropLens for GEO {
    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>, version: VcardVersion) -> VcardValue<'v> {
        line.decode_value(VcardPropKind::Geo, version)
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for GEO {
    const KIND: VcardPropKind = VcardPropKind::Geo;

    fn allowed_values(version: VcardVersion) -> &'static [VcardValueKind] {
        match version {
            VcardVersion::V4_0 => &[VcardValueKind::Uri],
            _ => &[VcardValueKind::Geo],
        }
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
