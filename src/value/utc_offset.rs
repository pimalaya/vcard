//! # UTC-offset value
//!
//! The decoded UTC-offset value kind.
//!
//! One of the forms the `TZ` property may take: a signed `hhmm` offset from UTC
//! (e.g. `-0500`). The offset is kept as its raw text; `TZ` may instead be plain
//! text or a URI, decoded as the corresponding kinds. Pure data, no escaping;
//! the owning property's wire name lives on [`crate::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// A decoded UTC-offset value (signed `hhmm`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardUtcOffset<'a>(pub Cow<'a, str>);
