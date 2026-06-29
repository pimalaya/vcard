//! # URI value
//!
//! The decoded URI value kind, backing the 2.1 `URL` property (a link to a
//! resource). The reference is kept verbatim; the crate does not parse or
//! validate it. (Binary properties that may also be a URI use
//! [`binary::VcardBinary`](crate::v2_1::value::binary::VcardBinary) instead, to
//! capture the inline-vs-reference choice.) The wire name lives on
//! [`crate::v2_1::prop::VcardProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded URI value, kept verbatim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardUri<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for VcardUri<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardUri<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardUri<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
