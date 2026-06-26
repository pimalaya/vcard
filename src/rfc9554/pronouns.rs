//! The PRONOUNS property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// PRONOUNS: the pronouns to use for the entity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardPronouns<'a> {
    /// The pronouns value.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardPronouns<'a>> for VcardExtension<'a> {
    fn from(property: VcardPronouns<'a>) -> Self {
        VcardExtension::single("PRONOUNS", property.params, property.value)
    }
}
