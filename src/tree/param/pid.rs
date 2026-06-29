//! # PID parameter lens
//!
//! The `PID` parameter lens: a list of source identifiers.

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::{
    param::VCARD_PID,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `PID` parameter lens.
pub struct PID;

impl VcardParamLens for PID {
    const NAME: &'static str = VCARD_PID;

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
            name: VcardLeaf::from(VCARD_PID.to_string()),
            values: decoded
                .iter()
                .map(|value| VcardLeaf::from(value.to_string()))
                .collect(),
        }
    }
}
