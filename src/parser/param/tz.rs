//! The TZ parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::tz::TZ;

/// The TZ parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct TZ {}

impl VcardParamLens for TZ {
    const NAME: &'static str = TZ;
}
