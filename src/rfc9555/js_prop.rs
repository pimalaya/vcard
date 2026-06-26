//! The JSPROP property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// JSPROP: a JSContact property carried as JSON text (its JSPTR parameter
/// points at the JSContact path until typed).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardJsProp<'a> {
    /// The JSON value, as written.
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardJsProp<'a>> for VcardExtension<'a> {
    fn from(property: VcardJsProp<'a>) -> Self {
        VcardExtension::single("JSPROP", property.params, property.value)
    }
}
