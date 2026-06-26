//! The GEO parameter.

use crate::parser::param::lens::VcardParamLens;
use crate::rfc6350::param::geo::GEO;

/// The GEO parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct GEO {}

impl VcardParamLens for GEO {
    const NAME: &'static str = GEO;
}
