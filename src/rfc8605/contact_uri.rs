//! The CONTACT-URI property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// CONTACT-URI: a URI for contacting the entity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardContactUri<'a> {
    /// The contact URI.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardContactUri<'a>> for VcardExtension<'a> {
    fn from(property: VcardContactUri<'a>) -> Self {
        VcardExtension::single("CONTACT-URI", property.params, property.value)
    }
}
