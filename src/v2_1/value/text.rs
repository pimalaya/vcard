//! # Text value
//!
//! The decoded text value kind: a single text value.
//!
//! Backs the 2.1 properties whose value is plain text: `FN`, `TITLE`, `ROLE`,
//! `NOTE`, `TEL`, `EMAIL`, `MAILER`, `LABEL`, `TZ`, `UID`, `AGENT`. Pure decoded
//! data; the 2.1 codec (`\;` unescaping plus optional `QUOTED-PRINTABLE`) lives
//! on the syntax side ([`crate::v2_1::tree`]). The wire name lives on
//! [`crate::v2_1::prop::VcardProp::name`].

use alloc::{borrow::Cow, string::String};

/// A single decoded text value.
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
