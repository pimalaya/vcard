//! The MEDIATYPE parameter.

use crate::{parser::param::lens::VcardParamLens, rfc6350::param::mediatype::MEDIATYPE};

/// The MEDIATYPE parameter as a type, for type-driven lookups.
#[allow(non_camel_case_types)]
pub struct MEDIATYPE {}

impl VcardParamLens for MEDIATYPE {
    const NAME: &'static str = MEDIATYPE;
}
