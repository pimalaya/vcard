//! # CALSCALE parameter lens
//!
//! The `CALSCALE` parameter lens: a single value (the calendar scale of a
//! date/time value) (RFC 6350 5.8).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::VcardParamKind,
    tree::{
        codec::{escape::escape_param, mode::VcardEscaper, unescape::unescape_param},
        leaf::VcardLeaf,
        param::{lens::VcardParamLens, node::VcardParamNode},
    },
};

/// The `CALSCALE` parameter lens.
pub struct CALSCALE;

impl VcardParamLens for CALSCALE {
    const KIND: VcardParamKind = VcardParamKind::CalScale;

    type Target<'v> = Cow<'v, str>;

    fn decode<'v>(param: &'v VcardParamNode<'_>) -> Cow<'v, str> {
        param
            .values
            .first()
            .map(|value| unescape_param(value.get(), param.escaper))
            .unwrap_or_default()
    }

    fn encode(decoded: &Cow<'_, str>, escaper: VcardEscaper) -> VcardParamNode<'static> {
        VcardParamNode {
            name: VcardLeaf::from(Self::KIND.to_string()),
            values: vec![VcardLeaf::from(escape_param(decoded, escaper).into_owned())],
            escaper,
        }
    }
}
