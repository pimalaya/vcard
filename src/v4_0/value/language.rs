//! # Language-tag value
//!
//! The decoded language-tag value kind.
//!
//! Backs the `LANG` property, whose value is an RFC 5646 language tag (e.g.
//! `en`, `fr-CA`). The tag is kept verbatim; the crate does not parse its
//! subtags. Pure data with no escaping, like the rest of [`crate::v4_0::value`]; the
//! owning property's wire name lives on [`crate::v4_0::prop::VcardProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded RFC 5646 language tag, kept verbatim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardLanguageTag<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for VcardLanguageTag<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardLanguageTag<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardLanguageTag<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
