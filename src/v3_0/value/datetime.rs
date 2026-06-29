//! # Date and time values
//!
//! The decoded time-related value kinds: a date-and-or-time, and a timestamp.
//!
//! [`VcardDateAndOrTime`] backs `BDAY` (a date or date-time); [`VcardTimestamp`]
//! backs `REV`. RFC 2426 date and date-time values follow the ISO 8601 calendar
//! grammar; rather than decode into broken calendar fields and risk a lossy
//! round-trip, the value is kept as its raw text. Callers that need calendar
//! semantics parse the string themselves. Pure data, no escaping; the owning
//! property's wire name lives on [`crate::v3_0::prop::VcardProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded date-and-or-time value, kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardDateAndOrTime<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for VcardDateAndOrTime<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardDateAndOrTime<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardDateAndOrTime<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

/// A decoded timestamp value, kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardTimestamp<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for VcardTimestamp<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardTimestamp<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardTimestamp<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
