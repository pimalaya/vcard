//! # Language-tag value
//!
//! The decoded language-tag value kind.
//!
//! Backs the `LANG` property, whose value is an RFC 5646 language tag (e.g.
//! `en`, `fr-CA`). The tag is kept verbatim; the crate does not parse its
//! subtags. Pure data with no escaping, like the rest of [`crate::value`]; the
//! owning property's wire name lives on [`crate::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// A decoded RFC 5646 language tag, kept verbatim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardLanguageTag<'a>(pub Cow<'a, str>);
