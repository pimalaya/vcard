//! # ENCODING parameter lens
//!
//! The `ENCODING` parameter lens: a single value (the inline value encoding,
//! `b` for base64).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::v3_0::{
    param::VCARD_ENCODING,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `ENCODING` parameter lens.
pub struct ENCODING;

impl VcardParamLens for ENCODING {
    const NAME: &'static str = VCARD_ENCODING;

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
            name: VcardLeaf::from(VCARD_ENCODING.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
