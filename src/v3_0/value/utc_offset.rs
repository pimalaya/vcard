//! # UTC-offset value
//!
//! The decoded UTC-offset value kind.
//!
//! The default form of the `TZ` property: a signed offset from UTC (e.g.
//! `-05:00`). The offset is kept as its raw text; `TZ` may instead be reset to
//! plain text, which round-trips through this same kind. Pure data, no escaping;
//! the owning property's wire name lives on [`crate::v3_0::prop::VcardProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded UTC-offset value (signed `hh:mm`), kept as its raw text.
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
