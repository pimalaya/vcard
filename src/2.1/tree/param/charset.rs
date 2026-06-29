//! # CHARSET parameter lens
//!
//! The `CHARSET` parameter lens: a single value (the value's character set).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::v21::{
    param::VCARD_CHARSET,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `CHARSET` parameter lens.
pub struct CHARSET;

impl VcardParamLens for CHARSET {
    const NAME: &'static str = VCARD_CHARSET;

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
            name: VcardLeaf::from(VCARD_CHARSET.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
