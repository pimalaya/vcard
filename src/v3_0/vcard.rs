//! # Card
//!
//! The decoded card: the entry point of the model.
//!
//! A [`Vcard`] is just a version and an ordered list of properties; the property,
//! value, parameter and version types each live in their own sibling module.
//! This module also owns the card-framing name vocabulary ([`VCARD`],
//! [`VCARD_BEGIN`], [`VCARD_END`]) used to recognise and emit the `BEGIN:VCARD` /
//! `END:VCARD` envelope. Like the rest of the decoded model it has no dependency
//! on [`crate::v3_0::tree`]; rendering a `Vcard` back to bytes is provided by a
//! [`Display`](core::fmt::Display) impl that lives on the syntax side.

use alloc::vec::Vec;

use crate::v3_0::{prop::VcardProp, version::VcardVersion};

pub const VCARD: &str = "VCARD";
pub const VCARD_BEGIN: &str = "BEGIN";
pub const VCARD_END: &str = "END";

/// A decoded card: its version and its properties, in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vcard<'a> {
    /// The card version.
    pub version: VcardVersion<'a>,
    /// The properties, in source order.
    pub properties: Vec<VcardProp<'a>>,
}
