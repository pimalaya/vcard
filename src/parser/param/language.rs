//! The LANGUAGE parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::language::LANGUAGE;

/// The LANGUAGE parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct LANGUAGE {}

impl VcardParamLens for LANGUAGE {
    const NAME: &'static str = LANGUAGE;
}
