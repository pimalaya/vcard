//! # Card
//!
//! The decoded card: the entry point of the model.
//!
//! A [`Vcard`] is just a version and an ordered list of properties; the
//! property, value, parameter and version types each live in their own sibling
//! module. This module owns [`VCARD_VERSION_30`], the wire string this version
//! emits in its `VERSION` line; the shared `BEGIN:VCARD` / `END:VCARD` framing
//! vocabulary lives at the crate root in [`crate::vcard`]. Like the rest of the
//! decoded model it has no dependency on [`crate::v30::tree`]; rendering a
//! `Vcard` back to bytes is provided by a [`Display`](core::fmt::Display) impl
//! that lives on the syntax side.

use alloc::vec::Vec;

use crate::v30::prop::VcardProp;
use crate::version::VcardVersion;

/// The vCard 3.0 wire string, emitted in this version's `VERSION` line.
pub const VCARD_VERSION_30: &str = "3.0";

/// A decoded card: its version and its properties, in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vcard<'a> {
    /// The card version.
    pub version: VcardVersion<'a>,
    /// The properties, in source order.
    pub properties: Vec<VcardProp<'a>>,
}
