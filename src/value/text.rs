//! # Text values
//!
//! The decoded text value kinds: a single text, and a comma-separated text list.
//!
//! These back the bulk of RFC 6350 properties whose value is plain text: `FN`,
//! `TITLE`, `ROLE`, `NOTE`, `PRODID`, `KIND`, `TEL`, `EMAIL`, ... for
//! [`VcardText`], and `NICKNAME` / `CATEGORIES` for [`VcardTextList`]. They are
//! pure, always-unescaped data; the escaping and the wire framing live entirely
//! on the syntax side ([`crate::tree`]), so the same value type round-trips
//! through any property that shares the kind. The wire name that distinguishes
//! those properties is carried by [`crate::prop::VcardProp::name`], not here.

use alloc::{borrow::Cow, vec::Vec};

/// A single decoded text value (unescaped).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardText<'a>(pub Cow<'a, str>);

/// A decoded comma-separated text list (each item unescaped).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardTextList<'a>(pub Vec<Cow<'a, str>>);
