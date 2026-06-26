//! The INTEREST property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// INTEREST: an interest the entity has but may not pursue.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardInterest<'a> {
    /// The interest value.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardInterest<'a>> for VcardExtension<'a> {
    fn from(property: VcardInterest<'a>) -> Self {
        VcardExtension::single("INTEREST", property.params, property.value)
    }
}
