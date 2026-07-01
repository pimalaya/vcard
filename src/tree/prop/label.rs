//! # LABEL lens
//!
//! The `LABEL` property lens: a formatted delivery-address label, decoded as a
//! single text value.
//!
//! See RFC 2426 3.2.2 (vCard 2.1/3.0; a parameter in 4.0).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{VcardPropLens, VcardPropSpec},
        value::VcardValueCursor,
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `LABEL` property lens.
pub struct LABEL;

impl VcardPropLens for LABEL {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for LABEL {
    const KIND: VcardPropKind = VcardPropKind::Label;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V2_1, VcardVersion::V3_0]
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
