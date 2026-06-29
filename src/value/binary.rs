//! # Binary value
//!
//! The decoded binary value kind.
//!
//! Backs the binary-bearing properties (`PHOTO`, `LOGO`, `SOUND`, `KEY`) in
//! vCard 2.1 and 3.0, where the value is either an external URI reference or
//! inline base64. vCard 4.0 carries these as `data:` URIs instead, decoded to
//! [`VcardUri`](crate::value::uri::VcardUri). The form is told by the line's
//! `VALUE` / `ENCODING` parameters; the payload is kept verbatim (base64 is not
//! decoded to bytes). Pure data with no escaping knowledge.

use alloc::borrow::Cow;

/// A decoded binary value: an external URI reference, or inline base64 kept as
/// its raw text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardBinary<'a> {
    /// An external URI reference.
    Uri(Cow<'a, str>),
    /// Inline base64 data, kept verbatim (not decoded to bytes).
    Base64(Cow<'a, str>),
}
