//! # PREF parameter lens
//!
//! The `PREF` parameter lens: a single value (a preference, 1-100).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::VCARD_PREF,
    tree::{decode::unescape, leaf::VcardLeaf, lens::VcardParamLens, param::VcardParamNode},
};

/// The `PREF` parameter lens.
pub struct PREF;

impl VcardParamLens for PREF {
    const NAME: &'static str = VCARD_PREF;

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
            name: VcardLeaf::from(VCARD_PREF.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
