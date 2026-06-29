//! # Binary value
//!
//! The decoded value kind for 2.1 properties that carry binary data: `PHOTO`,
//! `LOGO`, `SOUND`, `KEY`.
//!
//! Per the 2.1 spec, such a value is either an external reference (`VALUE=URL`)
//! or inline data (`ENCODING=BASE64`). This enum models both forms; which one a
//! line uses is decided from its parameters on the syntax side
//! ([`crate::v21::tree`]). The inline form is kept as its base64 text (decoding
//! to raw bytes is deferred to a feature). The wire name lives on
//! [`crate::v21::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// The decoded value of a binary property: an external URI or inline base64.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardBinary<'a> {
    /// An external reference (`VALUE=URL`).
    Uri(Cow<'a, str>),
    /// Inline data (`ENCODING=BASE64`), kept as its base64 text.
    Base64(Cow<'a, str>),
}
