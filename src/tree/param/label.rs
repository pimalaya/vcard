//! # LABEL parameter lens
//!
//! The `LABEL` parameter lens: a single value (the formatted text of a delivery
//! address), carried on `ADR` (RFC 6350 6.3.1).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::VcardLeaf;
use crate::{
    param::VcardParamKind,
    tree::{codec::unescape::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `LABEL` parameter lens.
pub struct LABEL;

impl VcardParamLens for LABEL {
    const KIND: VcardParamKind = VcardParamKind::Label;

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
