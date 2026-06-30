//! # AGENT lens
//!
//! The `AGENT` property lens: another entity acting as the contact's agent,
//! kept as opaque text and decoded as a single text value.
//!
//! See RFC 2426 3.5.4 (vCard 3.0; removed in 4.0).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        cursor::VcardValueCursor,
        line::VcardLine,
        prop::{VcardPropCardinality, VcardPropLens, VcardPropSpec},
        value::VcardValueNode,
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `AGENT` property lens.
pub struct AGENT;

impl VcardPropLens for AGENT {
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

impl VcardPropSpec for AGENT {
    const PROP: VcardPropKind = VcardPropKind::Agent;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V2_1, VcardVersion::V3_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Type,
            VcardParamKind::Language,
            VcardParamKind::Encoding,
            VcardParamKind::Charset,
            VcardParamKind::Value,
        ]
    }
}
