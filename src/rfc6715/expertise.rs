//! The EXPERTISE property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// EXPERTISE: a field of expertise.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardExpertise<'a> {
    /// The expertise value.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardExpertise<'a>> for VcardExtension<'a> {
    fn from(property: VcardExpertise<'a>) -> Self {
        VcardExtension::single("EXPERTISE", property.params, property.value)
    }
}
