//! The SORT_AS parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::sort_as::SORT_AS;

/// The SORT_AS parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct SORT_AS {}

impl VcardParamLens for SORT_AS {
    const NAME: &'static str = SORT_AS;
}
