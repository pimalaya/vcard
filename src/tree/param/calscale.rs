//! # CALSCALE parameter lens
//!
//! The `CALSCALE` parameter lens: a single value (the calendar scale of a
//! date/time value) (RFC 6350 5.8).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::VcardParamKind,
    tree::{
        codec::unescape::unescape,
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
            .map(|value| unescape(value.get()))
            .unwrap_or_default()
    }

    fn encode(decoded: &Cow<'_, str>) -> VcardParamNode<'static> {
        VcardParamNode {
            name: VcardLeaf::from(Self::KIND.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
