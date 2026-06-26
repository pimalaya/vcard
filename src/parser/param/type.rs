//! The TYPE parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::r#type::TYPE;

/// The TYPE parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct TYPE {}

impl VcardParamLens for TYPE {
    const NAME: &'static str = TYPE;
}
