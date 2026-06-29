//! # CALSCALE parameter lens
//!
//! The `CALSCALE` parameter lens: a single value (the calendar scale of a
//! date/time value).

use alloc::{borrow::Cow, string::ToString, vec};

use crate::v4_0::{
    param::VCARD_CALSCALE,
    tree::{decode::unescape, leaf::VcardLeaf, param::VcardParamLens, param::VcardParamNode},
};

/// The `CALSCALE` parameter lens.
pub struct CALSCALE;

impl VcardParamLens for CALSCALE {
    const NAME: &'static str = VCARD_CALSCALE;

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
            name: VcardLeaf::from(VCARD_CALSCALE.to_string()),
            values: vec![VcardLeaf::from(decoded.to_string())],
        }
    }
}
