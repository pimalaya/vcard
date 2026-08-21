//! # vCard
//!
//! The decoded card: a version indicator and its properties, in source order.
//!
//! The property, value, parameter and version types live in the sibling
//! modules. Like the rest of the decoded model, [`Vcard`] has no dependency on
//! [`crate::tree`]; rendering it back to bytes is a
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
