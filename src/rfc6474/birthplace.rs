//! The BIRTHPLACE property.

use alloc::vec::Vec;

use crate::rfc6350::{
    extension::VcardExtension, param::parameter::VcardParameter, value::VcardUriOrText,
};

/// BIRTHPLACE: the birthplace, as free text or a URI.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardBirthplace<'a> {
    /// The birthplace value.
    pub value: VcardUriOrText<'a>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardBirthplace<'a>> for VcardExtension<'a> {
    fn from(property: VcardBirthplace<'a>) -> Self {
        VcardExtension::uri_or_text("BIRTHPLACE", property.params, property.value)
    }
}
