//! # PROFILE lens
//!
//! The `PROFILE` property lens: a vCard 3.0 declaration (value `VCARD`) stating
//! the card conforms to the vCard profile, removed in 4.0. The lens decodes it
//! as a single `VcardText` (RFC 2426 3.1.4).

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        prop::{cardinality::VcardPropCardinality, lens::VcardPropLens, spec::VcardPropSpec},
        value::cursor::VcardValueCursor,
    },
    value::text::VcardText,
    version::VcardVersion,
};

/// The `PROFILE` property lens.
pub struct PROFILE;

impl VcardPropLens for PROFILE {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for PROFILE {
    const KIND: VcardPropKind = VcardPropKind::Profile;

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
