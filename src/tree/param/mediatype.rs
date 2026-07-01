//! # MEDIATYPE parameter lens
//!
//! The `MEDIATYPE` parameter lens: a single value (the media type of a
//! referenced resource) (RFC 6350 5.7).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::tree::leaf::VcardLeaf;
use crate::{
    param::VcardParamKind,
    tree::{codec::unescape::unescape, param::VcardParamLens, param::VcardParamNode},
};

/// The `MEDIATYPE` parameter lens.
pub struct MEDIATYPE;

impl VcardParamLens for MEDIATYPE {
    const KIND: VcardParamKind = VcardParamKind::MediaType;

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
