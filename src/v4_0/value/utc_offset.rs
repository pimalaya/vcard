//! # UTC-offset value
//!
//! The decoded UTC-offset value kind.
//!
//! One of the forms the `TZ` property may take: a signed `hhmm` offset from UTC
//! (e.g. `-0500`). The offset is kept as its raw text; `TZ` may instead be plain
//! text or a URI, decoded as the corresponding kinds. Pure data, no escaping;
//! the owning property's wire name lives on [`crate::v4_0::prop::VcardProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded UTC-offset value (signed `hhmm`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardUtcOffset<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for VcardUtcOffset<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardUtcOffset<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardUtcOffset<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
