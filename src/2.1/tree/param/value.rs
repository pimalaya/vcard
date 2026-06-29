//! # VALUE parameter lens
//!
//! The `VALUE` parameter lens: a single value naming the value type to read the
//! property value as.

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::VcardLeaf;
use crate::v21::{
    param::VCARD_VALUE,
    tree::{decode::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `VALUE` parameter lens.
pub struct VALUE;

impl VcardParamLens for VALUE {
    const NAME: &'static str = VCARD_VALUE;

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
            name: VcardLeaf::from(VCARD_VALUE.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
