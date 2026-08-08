//! # GEO parameter lens
//!
//! The `GEO` parameter lens: a single value (a global positioning value for the
//! property) (RFC 6350 5.10). Distinct from the `GEO` property.

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::VcardParamKind,
    tree::{
        codec::unescape::unescape,
        leaf::VcardLeaf,
        param::{lens::VcardParamLens, node::VcardParamNode},
    },
};

/// The `GEO` parameter lens.
pub struct GEO;

impl VcardParamLens for GEO {
    const KIND: VcardParamKind = VcardParamKind::Geo;

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
