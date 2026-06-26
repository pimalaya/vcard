//! The LANGUAGE property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// LANGUAGE: the preferred language for contacting the entity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardLanguage<'a> {
    /// The language-tag value.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardLanguage<'a>> for VcardExtension<'a> {
    fn from(property: VcardLanguage<'a>) -> Self {
        VcardExtension::single("LANGUAGE", property.params, property.value)
    }
}
