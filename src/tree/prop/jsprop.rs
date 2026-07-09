//! # JSPROP lens
//!
//! The `JSPROP` property lens: a JSContact property with no vCard
//! counterpart, preserved as JSON text at the position named by its `JSPTR`
//! parameter.
//!
//! See RFC 9555.

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

/// The `JSPROP` property lens.
pub struct JSPROP;

impl VcardPropLens for JSPROP {
    type Target<'v> = VcardText<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}

impl VcardPropSpec for JSPROP {
    const KIND: VcardPropKind = VcardPropKind::JsProp;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[VcardParamKind::Jsptr, VcardParamKind::Value]
    }
}
