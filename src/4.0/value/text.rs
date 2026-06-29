//! # Text values
//!
//! The decoded text value kinds: a single text, and a comma-separated text list.
//!
//! These back the bulk of RFC 6350 properties whose value is plain text: `FN`,
//! `TITLE`, `ROLE`, `NOTE`, `PRODID`, `KIND`, `TEL`, `EMAIL`, ... for
//! [`VcardText`], and `NICKNAME` / `CATEGORIES` for [`VcardTextList`]. They are
//! pure, always-unescaped data; the escaping and the wire framing live entirely
//! on the syntax side ([`crate::v40::tree`]), so the same value type round-trips
//! through any property that shares the kind. The wire name that distinguishes
//! those properties is carried by [`crate::v40::prop::VcardProp::name`], not here.

use alloc::{borrow::Cow, string::String, vec::Vec};

/// A single decoded text value (unescaped).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardText<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for VcardText<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardText<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardText<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

/// A decoded comma-separated text list (each item unescaped).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardTextList<'a>(pub Vec<Cow<'a, str>>);

impl<'a> From<Vec<Cow<'a, str>>> for VcardTextList<'a> {
    fn from(values: Vec<Cow<'a, str>>) -> Self {
        Self(values)
    }
}
