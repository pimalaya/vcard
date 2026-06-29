//! # MEDIATYPE parameter lens
//!
//! The `MEDIATYPE` parameter lens: a single value (the media type of a
//! referenced resource).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::v4_0::{
    param::VCARD_MEDIATYPE,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `MEDIATYPE` parameter lens.
pub struct MEDIATYPE;

impl VcardParamLens for MEDIATYPE {
    const NAME: &'static str = VCARD_MEDIATYPE;

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
            name: VcardLeaf::from(VCARD_MEDIATYPE.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
