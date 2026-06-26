//! The CALSCALE parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::calscale::CALSCALE;

/// The CALSCALE parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct CALSCALE {}

impl VcardParamLens for CALSCALE {
    const NAME: &'static str = CALSCALE;
}
