//! # Binary value
//!
//! The decoded value kind for RFC 2426 properties that carry binary data:
//! `PHOTO`, `LOGO`, `SOUND`, `KEY`.
//!
//! Such a value is either an external reference (`VALUE=uri`) or inline data
//! (`ENCODING=b`, base64). Both forms are a single run of text, so the value is
//! kept verbatim in one [`Cow`]; which form a line uses is told by its `VALUE` /
//! `ENCODING` parameters (decoded into
//! [`VcardParam`](crate::v30::param::VcardParam)). The inline form is kept as
//! its base64 text; decoding to raw bytes is left to the caller. Pure data, no
//! escaping; the owning property's wire name lives on
//! [`crate::v30::prop::VcardProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded binary value, kept verbatim: an external URI, or inline base64.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardBinary<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for VcardBinary<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardBinary<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardBinary<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
