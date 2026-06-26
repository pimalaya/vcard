//! The HOBBY property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// HOBBY: a hobby actively pursued.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardHobby<'a> {
    /// The hobby value.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardHobby<'a>> for VcardExtension<'a> {
    fn from(property: VcardHobby<'a>) -> Self {
        VcardExtension::single("HOBBY", property.params, property.value)
    }
}
