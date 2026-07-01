//! # Codec
//!
//! The bytes-to-model bridge, in both directions and at both levels. It is the
//! only part of [`crate::tree`] that consults the card version.
//!
//! [`decode`] projects a raw syntax tree onto the decoded model and [`encode`]
//! projects it back; that is the structural level. Underneath, the value-string
//! level: [`escape`] and [`unescape`] apply and resolve the RFC 6350 3.4 value
//! escapes (keyed by the [`mode`] `Escaper`), and [`quoted_printable`] decodes
//! the 2.1 `=XX` octet encoding. The structural encoders and decoders run every
//! value leaf through those.

pub mod decode;
pub mod encode;
pub mod escape;
pub mod mode;
pub mod quoted_printable;
pub mod unescape;
pub mod value;
