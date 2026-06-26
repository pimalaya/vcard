//! The ALTID parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::altid::ALTID;

/// The ALTID parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct ALTID {}

impl VcardParamLens for ALTID {
    const NAME: &'static str = ALTID;
}
