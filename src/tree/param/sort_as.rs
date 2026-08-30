//! # SORT-AS parameter lens
//!
//! The `SORT-AS` parameter lens: a list of components to sort the property by
//! (RFC 6350 5.9).

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::{
    param::VcardParamKind,
    tree::{
        codec::{escape::escape_param, mode::VcardEscaper, unescape::unescape_param},
        leaf::VcardLeaf,
        param::{lens::VcardParamLens, node::VcardParamNode},
    },
};

/// The `SORT-AS` parameter lens.
#[allow(non_camel_case_types)]
pub struct SORT_AS;

impl VcardParamLens for SORT_AS {
    const KIND: VcardParamKind = VcardParamKind::SortAs;

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
