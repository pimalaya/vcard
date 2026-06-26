//! The DEATHPLACE property.

use alloc::vec::Vec;

use crate::rfc6350::{
    extension::VcardExtension, param::parameter::VcardParameter, value::VcardUriOrText,
};

/// DEATHPLACE: the place of death, as free text or a URI.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardDeathplace<'a> {
    /// The death-place value.
    pub value: VcardUriOrText<'a>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardDeathplace<'a>> for VcardExtension<'a> {
    fn from(property: VcardDeathplace<'a>) -> Self {
        VcardExtension::uri_or_text("DEATHPLACE", property.params, property.value)
    }
}
