//! The DEATHDATE property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// DEATHDATE: the date of death, as a date-and-or-time or free text value.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardDeathdate<'a> {
    /// The death-date value, as written.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardDeathdate<'a>> for VcardExtension<'a> {
    fn from(property: VcardDeathdate<'a>) -> Self {
        VcardExtension::single("DEATHDATE", property.params, property.value)
    }
}
