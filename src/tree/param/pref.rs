//! # PREF parameter lens
//!
//! The `PREF` parameter lens: a single value (a preference, 1-100).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::VcardLeaf;
use crate::{
    param::VcardParamKind,
    tree::{decode::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `PREF` parameter lens.
pub struct PREF;

impl VcardParamLens for PREF {
    const KIND: VcardParamKind = VcardParamKind::Pref;

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
