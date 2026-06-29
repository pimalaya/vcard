//! # LANGUAGE parameter lens
//!
//! The `LANGUAGE` parameter lens: a single value (an RFC 5646 language tag).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::v4_0::{
    param::VCARD_LANGUAGE,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `LANGUAGE` parameter lens.
pub struct LANGUAGE;

impl VcardParamLens for LANGUAGE {
    const NAME: &'static str = VCARD_LANGUAGE;

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
            name: VcardLeaf::from(VCARD_LANGUAGE.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
