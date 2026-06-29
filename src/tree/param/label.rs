//! # LABEL parameter lens
//!
//! The `LABEL` parameter lens: a single value (the formatted text of a delivery
//! address).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::VCARD_LABEL,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `LABEL` parameter lens.
pub struct LABEL;

impl VcardParamLens for LABEL {
    const NAME: &'static str = VCARD_LABEL;

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
            name: VcardLeaf::from(VCARD_LABEL.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
