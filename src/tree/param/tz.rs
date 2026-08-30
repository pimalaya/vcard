//! # TZ parameter lens
//!
//! The `TZ` parameter lens: a single value (the time zone of the property)
//! (RFC 6350 5.11). Distinct from the `TZ` property.

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::VcardParamKind,
    tree::{
        codec::{escape::escape_param, mode::VcardEscaper, unescape::unescape_param},
        leaf::VcardLeaf,
        param::{lens::VcardParamLens, node::VcardParamNode},
    },
};

/// The `TZ` parameter lens.
pub struct TZ;

impl VcardParamLens for TZ {
    const KIND: VcardParamKind = VcardParamKind::Tz;

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
