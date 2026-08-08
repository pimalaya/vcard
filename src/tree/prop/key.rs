//! # KEY lens
//!
//! The `KEY` property lens. Its value shape is version-specific, so the lens
//! decodes through the card version: a `data:` URI in 4.0
//! ([`VcardValue::Uri`]), inline base64 or a URI reference in 2.1 / 3.0
//! ([`VcardValue::Binary`]). The lens overrides
//! [`decode`](VcardPropLens::decode) to resolve the shape from the version
//! through the property spec.
//!
//! See RFC 6350 6.8.1.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{lens::VcardPropLens, spec::VcardPropSpec},
        value::cursor::VcardValueCursor,
    },
    value::{VcardValue, VcardValueKind},
    version::VcardVersion,
};

/// The `KEY` property lens.
pub struct KEY;

impl VcardPropLens for KEY {
    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>, version: VcardVersion) -> VcardValue<'v> {
        line.decode_value(VcardPropKind::Key, version)
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for KEY {
    const KIND: VcardPropKind = VcardPropKind::Key;

    fn allowed_values(version: VcardVersion) -> &'static [VcardValueKind] {
        match version {
            VcardVersion::V4_0 => &[VcardValueKind::Uri],
            _ => &[VcardValueKind::Binary, VcardValueKind::Uri],
        }
    }

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
