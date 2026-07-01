//! # ALTID parameter lens
//!
//! The `ALTID` parameter lens: a single value tying alternative representations
//! of the same logical property (RFC 6350 5.4).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::VcardLeaf;
use crate::{
    param::VcardParamKind,
    tree::{codec::unescape::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `ALTID` parameter lens.
pub struct ALTID;

impl VcardParamLens for ALTID {
    const KIND: VcardParamKind = VcardParamKind::AltId;

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
