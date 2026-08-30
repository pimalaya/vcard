//! # TYPE parameter lens
//!
//! The `TYPE` parameter lens: a list of kinds or contexts (e.g. `work`, `home`)
//! (RFC 6350 5.6).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::{
    param::VcardParamKind,
    tree::{
        codec::{escape::escape_param, mode::VcardEscaper, unescape::unescape_param},
        leaf::VcardLeaf,
        param::{lens::VcardParamLens, node::VcardParamNode},
    },
};

/// The `TYPE` parameter lens.
pub struct TYPE;

impl VcardParamLens for TYPE {
    const KIND: VcardParamKind = VcardParamKind::Type;

    type Target<'v> = Vec<Cow<'v, str>>;

    fn decode<'v>(param: &'v VcardParamNode<'_>) -> Vec<Cow<'v, str>> {
        param
            .values
            .iter()
            .map(|value| unescape_param(value.get(), param.escaper))
            .collect()
    }

    #[allow(clippy::ptr_arg)]
    fn encode(decoded: &Vec<Cow<'_, str>>, escaper: VcardEscaper) -> VcardParamNode<'static> {
        VcardParamNode {
            name: VcardLeaf::from(Self::KIND.to_string()),
            values: decoded
                .iter()
                .map(|value| VcardLeaf::from(escape_param(value, escaper).into_owned()))
                .collect(),
            escaper,
        }
    }
}
