//! # URI value
//!
//! The decoded URI value kind.
//!
//! Backs every RFC 6350 property whose value is a URI: `SOURCE`, `PHOTO`,
//! `IMPP`, `LOGO`, `MEMBER`, `RELATED`, `SOUND`, `UID`, `KEY`, `GEO`, `URL`,
//! `FBURL`, `CALADRURI`, `CALURI`. The reference is kept verbatim as a string;
//! the crate does not parse or validate it. Pure data with no escaping
//! knowledge, like every other type in [`crate::v4_0::value`]; the owning property's
//! wire name lives on [`crate::v4_0::prop::VcardProp::name`].

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
