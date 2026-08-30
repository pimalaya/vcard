//! # PREF parameter lens
//!
//! The `PREF` parameter lens: a single value (a preference, 1-100)
//! (RFC 6350 5.3).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::{
    param::VcardParamKind,
    tree::{
        codec::{escape::escape_param, mode::VcardEscaper, unescape::unescape_param},
        leaf::VcardLeaf,
        param::{lens::VcardParamLens, node::VcardParamNode},
    },
};

/// The `PREF` parameter lens.
pub struct PREF;

impl VcardParamLens for PREF {
    const KIND: VcardParamKind = VcardParamKind::Pref;

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
