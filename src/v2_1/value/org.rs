//! # ORG value
//!
//! The decoded `ORG` (organization) value.
//!
//! In vCard 2.1, `ORG` is a `;`-ordered sequence of organization units, from the
//! top-level organization name down through divisions. This bespoke type holds
//! them as an ordered list. Pure decoded data; the `;` splitting and `\;`
//! escaping live on the syntax side ([`crate::v2_1::tree`]). The wire name lives
//! on [`crate::v2_1::prop::VcardProp::name`].

use alloc::{borrow::Cow, vec::Vec};

/// The decoded ORG value: organization units, broadest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardOrg<'a>(pub Vec<Cow<'a, str>>);

impl<'a> From<Vec<Cow<'a, str>>> for VcardOrg<'a> {
    fn from(units: Vec<Cow<'a, str>>) -> Self {
        Self(units)
    }
}
