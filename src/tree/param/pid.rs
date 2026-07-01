//! # PID parameter lens
//!
//! The `PID` parameter lens: a list of source identifiers (RFC 6350 5.5).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::tree::leaf::VcardLeaf;
use crate::{
    param::VcardParamKind,
    tree::{codec::unescape::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `PID` parameter lens.
pub struct PID;

impl VcardParamLens for PID {
    const KIND: VcardParamKind = VcardParamKind::Pid;

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
            name: VcardLeaf::from(Self::KIND.to_string()),
            values: decoded
                .iter()
                .map(|value| VcardLeaf::from(value.to_string()))
                .collect(),
        }
    }
}
