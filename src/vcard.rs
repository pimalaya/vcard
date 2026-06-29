//! # Card
//!
//! The decoded card and the wire names that frame it.
//!
//! A [`Vcard`] is just a version indicator and an ordered list of properties;
//! the property, value, parameter and version types each live in their own
//! sibling module. This module also owns the framing vocabulary ([`VCARD`],
//! [`VCARD_BEGIN`], [`VCARD_END`]) used to recognise and emit the `BEGIN:VCARD`
//! / `END:VCARD` envelope. Like the rest of the decoded model it has no
//! dependency on [`crate::tree`]; rendering a `Vcard` back to bytes is provided
//! by a [`Display`](core::fmt::Display) impl that lives on the syntax side.

use alloc::vec::Vec;

use crate::prop::VcardProp;
use crate::version::VcardVersion;

/// The vCard object name, the value of the `BEGIN` and `END` lines.
pub const VCARD: &str = "VCARD";
/// The name of the line that opens a card.
pub const VCARD_BEGIN: &str = "BEGIN";
/// The name of the line that closes a card.
pub const VCARD_END: &str = "END";

/// A decoded card: its version and its properties, in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vcard<'a> {
    /// The card version.
    pub version: VcardVersion<'a>,
    /// The properties, in source order.
    pub properties: Vec<VcardProp<'a>>,
}
