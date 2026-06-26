//! The CREATED property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// CREATED: the timestamp at which the card was created, as written.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardCreated<'a> {
    /// The creation timestamp value, as written.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardCreated<'a>> for VcardExtension<'a> {
    fn from(property: VcardCreated<'a>) -> Self {
        VcardExtension::single("CREATED", property.params, property.value)
    }
}
