//! The SOCIALPROFILE property.

use alloc::vec::Vec;

use crate::rfc6350::{
    extension::VcardExtension, param::parameter::VcardParameter, value::VcardUriOrText,
};

/// SOCIALPROFILE: a social-media profile, as a URI or free text (its
/// SERVICE-TYPE and USERNAME parameters live in the parameters until typed).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardSocialProfile<'a> {
    /// The profile value.
    pub value: VcardUriOrText<'a>,
    /// The parameters decorating it.
    pub params: Vec<VcardParameter<'a>>,
}

impl<'a> From<VcardSocialProfile<'a>> for VcardExtension<'a> {
    fn from(property: VcardSocialProfile<'a>) -> Self {
        VcardExtension::uri_or_text("SOCIALPROFILE", property.params, property.value)
    }
}
