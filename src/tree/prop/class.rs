//! # CLASS lens
//!
//! The `CLASS` property lens: the access classification of the card, decoded as
//! a single text value.
//!
//! See RFC 2426 3.7.1 (vCard 3.0; removed in 4.0).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{VcardPropCardinality, VcardPropLens, VcardPropSpec},
        value::VcardValueCursor,
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `CLASS` property lens.
pub struct CLASS;

impl VcardPropLens for CLASS {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for CLASS {
    const KIND: VcardPropKind = VcardPropKind::Class;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}
