//! # SORT-AS parameter lens
//!
//! The `SORT-AS` parameter lens: a list of components to sort the property by.

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::v2_1::{
    param::VCARD_SORT_AS,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `SORT-AS` parameter lens.
#[allow(non_camel_case_types)]
pub struct SORT_AS;

impl VcardParamLens for SORT_AS {
    const NAME: &'static str = VCARD_SORT_AS;

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
            name: VcardLeaf::from(VCARD_SORT_AS.to_string()),
            values: decoded
                .iter()
                .map(|value| VcardLeaf::from(value.to_string()))
                .collect(),
        }
    }
}
