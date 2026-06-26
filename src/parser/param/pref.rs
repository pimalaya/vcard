//! The PREF parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::pref::PREF;

/// The PREF parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct PREF {}

impl VcardParamLens for PREF {
    const NAME: &'static str = PREF;
}
