//! # PRODID lens
//!
//! The `PRODID` property lens: the identifier of the product that created the
//! card, decoded as a single `VcardText` (RFC 6350 6.7.3).

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

/// The `PRODID` property lens.
pub struct PRODID;

impl VcardPropLens for PRODID {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for PRODID {
    const KIND: VcardPropKind = VcardPropKind::ProdId;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V3_0, VcardVersion::V4_0]
    }

    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::AtMostOne
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Value]
    }
}
