//! # vCard
//!
//! The decoded card and the wire names that frame it.
//!
//! A [`Vcard`] is just a version indicator and an ordered list of properties;
//! the property, value, parameter and version types each live in their own
//! sibling module. Like the rest of the decoded model it has no dependency on
//! [`crate::tree`]; rendering a `Vcard` back to bytes is provided by a
//! [`Display`](core::fmt::Display) impl that lives on the syntax side.

use alloc::vec::Vec;

use crate::{prop::VcardProp, version::VcardVersion};

/// A decoded card: its version and its properties, in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vcard<'a> {
    /// The card version.
    pub version: VcardVersion,
    /// The properties, in source order.
    pub properties: Vec<VcardProp<'a>>,
}
