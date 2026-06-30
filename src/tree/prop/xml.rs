//! # XML lens
//!
//! The `XML` property lens: embedded XML, as a text value.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
        value::VcardValueNode,
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `XML` property lens.
pub struct XML;

impl VcardPropLens for XML {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardText<'v> {
        VcardText::decode(value)
    }

    fn encode(decoded: &VcardText<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for XML {
    const PROP: VcardPropKind = VcardPropKind::Xml;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::AltId, VcardParamKind::Value]
    }
}
