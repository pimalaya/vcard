//! # TYPE parameter lens
//!
//! The `TYPE` parameter lens: a list of kinds or contexts (e.g. `work`, `home`).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::tree::leaf::VcardLeaf;
use crate::{
    param::VCARD_TYPE,
    tree::{decode::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `TYPE` parameter lens.
pub struct TYPE;

impl VcardParamLens for TYPE {
    const NAME: &'static str = VCARD_TYPE;

    type Target<'v> = Vec<Cow<'v, str>>;

    fn decode<'v>(param: &'v VcardParamNode<'_>) -> Vec<Cow<'v, str>> {
        param
            .values
            .iter()
            .map(|value| unescape(value.get()))
            .collect()
    }

    #[allow(clippy::ptr_arg)]
    fn encode(decoded: &Vec<Cow<'_, str>>) -> VcardParamNode<'static> {
        VcardParamNode {
            name: VcardLeaf::from(VCARD_TYPE.to_string()),
            values: decoded
                .iter()
                .map(|value| VcardLeaf::from(value.to_string()))
                .collect(),
        }
    }
}
