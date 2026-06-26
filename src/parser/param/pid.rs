//! The PID parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::pid::PID;

/// The PID parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct PID {}

impl VcardParamLens for PID {
    const NAME: &'static str = PID;
}
