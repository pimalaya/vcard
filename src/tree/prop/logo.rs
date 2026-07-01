//! # LOGO lens
//!
//! The `LOGO` property lens. Its value shape is version-specific, so the lens
//! decodes through the card version: a `data:` URI in 4.0
//! ([`VcardValue::Uri`]), inline base64 or a URI reference in 2.1 / 3.0
//! ([`VcardValue::Binary`]). The lens overrides
//! [`decode`](VcardPropLens::decode) to resolve the shape from the version
//! through the property spec.
//!
//! See RFC 6350 6.6.3.

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

/// The `LOGO` property lens.
pub struct LOGO;

impl VcardPropLens for LOGO {
    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>, version: VcardVersion) -> VcardValue<'v> {
        line.decode_value(VcardPropKind::Logo, version)
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for LOGO {
    const KIND: VcardPropKind = VcardPropKind::Logo;

    fn allowed_values(version: VcardVersion) -> &'static [VcardValueKind] {
        match version {
            VcardVersion::V4_0 => &[VcardValueKind::Uri],
            _ => &[VcardValueKind::Binary, VcardValueKind::Uri],
        }
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Language,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::AltId,
            VcardParamKind::Value,
        ]
    }
}
