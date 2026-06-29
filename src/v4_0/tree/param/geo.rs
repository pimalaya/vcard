//! # GEO parameter lens
//!
//! The `GEO` parameter lens: a single value (a global positioning value for the
//! property). Distinct from the `GEO` property.

use alloc::{borrow::Cow, string::ToString, vec};

use crate::v4_0::{
    param::VCARD_GEO,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `GEO` parameter lens.
pub struct GEO;

impl VcardParamLens for GEO {
    const NAME: &'static str = VCARD_GEO;

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
            name: VcardLeaf::from(VCARD_GEO.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
