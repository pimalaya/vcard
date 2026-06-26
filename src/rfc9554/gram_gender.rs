//! The GRAMGENDER property.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::{extension::VcardExtension, param::parameter::VcardParameter};

/// GRAMGENDER: the grammatical gender to use in salutations.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardGramGender<'a> {
    /// The grammatical-gender value (animate, common, feminine, ...).
    pub value: Cow<'a, str>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardGramGender<'a>> for VcardExtension<'a> {
    fn from(property: VcardGramGender<'a>) -> Self {
        VcardExtension::single("GRAMGENDER", property.params, property.value)
    }
}
