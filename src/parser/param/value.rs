//! The VALUE parameter.

use crate::{parser::param::lens::VcardParamLens, rfc6350::param::value::VALUE};

/// The VALUE parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct VALUE {}

impl VcardParamLens for VALUE {
    const NAME: &'static str = VALUE;
}
