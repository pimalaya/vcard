//! The ORG-DIRECTORY property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// ORG-DIRECTORY: a URI for the organisation directory.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardOrgDirectory<'a> {
    /// The directory URI.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardOrgDirectory<'a>> for VcardExtension<'a> {
    fn from(property: VcardOrgDirectory<'a>) -> Self {
        VcardExtension::single("ORG-DIRECTORY", property.params, property.value)
    }
}
