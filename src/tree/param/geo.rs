//! # GEO parameter lens
//!
//! The `GEO` parameter lens: a single value (a global positioning value for the
//! property) (RFC 6350 5.10). Distinct from the `GEO` property.

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::VcardParamKind,
    tree::{
        codec::{escape::escape_param, mode::VcardEscaper, unescape::unescape_param},
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
            .map(|value| unescape_param(value.get(), param.escaper))
            .unwrap_or_default()
    }

    fn encode(decoded: &Cow<'_, str>, escaper: VcardEscaper) -> VcardParamNode<'static> {
        VcardParamNode {
            name: VcardLeaf::from(Self::KIND.to_string()),
            values: vec![VcardLeaf::from(escape_param(decoded, escaper).into_owned())],
            escaper,
        }
    }
}
