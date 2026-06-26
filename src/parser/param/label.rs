//! The LABEL parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::label::LABEL;

/// The LABEL parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct LABEL {}

impl VcardParamLens for LABEL {
    const NAME: &'static str = LABEL;
}
