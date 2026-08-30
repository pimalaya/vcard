//! # SOUND lens
//!
//! The `SOUND` property lens: a digital sound associated with the object, such
//! as a name pronunciation.
//!
//! Its value shape is version-specific, so the lens overrides
//! [`decode`](VcardPropLens::decode) to resolve it from the card version
//! through the property spec: a `data:` URI in 4.0 ([`VcardValue::Uri`]),
//! inline base64 or a URI reference in 2.1 / 3.0 ([`VcardValue::Binary`]).
//!
//! See RFC 6350 6.7.5.

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

/// The `SOUND` property lens.
pub struct SOUND;

impl VcardPropLens for SOUND {
    type Target<'v> = VcardValue<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>, version: VcardVersion) -> VcardValue<'v> {
        line.decode_value(VcardPropKind::Sound, version)
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for SOUND {
    const KIND: VcardPropKind = VcardPropKind::Sound;

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
