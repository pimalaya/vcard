//! # LANGUAGE parameter lens
//!
//! The `LANGUAGE` parameter lens: a single value (an RFC 5646 language tag)
//! (RFC 6350 5.1).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::VcardLeaf;
use crate::{
    param::VcardParamKind,
    tree::{decode::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `LANGUAGE` parameter lens.
pub struct LANGUAGE;

impl VcardParamLens for LANGUAGE {
    const KIND: VcardParamKind = VcardParamKind::Language;

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
