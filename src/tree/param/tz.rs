//! # TZ parameter lens
//!
//! The `TZ` parameter lens: a single value (the time zone of the property).
//! Distinct from the `TZ` property.

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::VcardLeaf;
use crate::{
    param::VcardParamKind,
    tree::{decode::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `TZ` parameter lens.
pub struct TZ;

impl VcardParamLens for TZ {
    const KIND: VcardParamKind = VcardParamKind::Tz;

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
