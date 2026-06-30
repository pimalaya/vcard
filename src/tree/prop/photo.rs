//! # PHOTO lens
//!
//! The `PHOTO` property lens. Its value shape is version-specific, so the lens
//! decodes through the card version: a `data:` URI in 4.0
//! ([`VcardValue::Uri`]), inline base64 or a URI reference in 2.1 / 3.0
//! ([`VcardValue::Binary`]). The version-blind
//! [`decode`](VcardPropLens::decode) assumes the 4.0 URI form.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
        value::VcardValueNode,
    },
    value::{VcardValue, VcardValueKind, uri::VcardUri},
    version::VcardVersion,
};

/// The `PHOTO` property lens.
pub struct PHOTO;

impl VcardPropLens for PHOTO {
    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardValue<'v> {
        VcardValue::Uri(VcardUri::decode(value))
    }

    fn decode_versioned<'v>(line: &'v VcardLine<'_>, version: VcardVersion) -> VcardValue<'v> {
        line.decode_value(VcardPropKind::Photo, version)
    }

    fn encode(decoded: &VcardValue<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for PHOTO {
    const PROP: VcardPropKind = VcardPropKind::Photo;

    fn allowed_values(version: VcardVersion) -> &'static [VcardValueKind] {
        match version {
            VcardVersion::V4_0 => &[VcardValueKind::Uri],
            _ => &[VcardValueKind::Binary, VcardValueKind::Uri],
        }
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::AltId,
            VcardParamKind::Type,
            VcardParamKind::MediaType,
            VcardParamKind::Pref,
            VcardParamKind::Pid,
            VcardParamKind::Value,
        ]
    }
}
