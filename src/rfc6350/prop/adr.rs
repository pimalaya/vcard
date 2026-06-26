//! The ADR property (RFC 6350 section 6.3.1).

use alloc::{borrow::Cow, vec::Vec};

/// The ADR property name.
pub const ADR: &str = "ADR";

/// A structured delivery address (the ADR property).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardAddress<'a> {
    /// Post office box (deprecated, kept for round-tripping).
    pub po_box: Vec<Cow<'a, str>>,
    /// Extended address such as an apartment or suite (deprecated).
    pub extended: Vec<Cow<'a, str>>,
    /// Street address.
    pub street: Vec<Cow<'a, str>>,
    /// Locality (city).
    pub locality: Vec<Cow<'a, str>>,
    /// Region (state or province).
    pub region: Vec<Cow<'a, str>>,
    /// Postal code.
    pub postal_code: Vec<Cow<'a, str>>,
    /// Country name.
    pub country: Vec<Cow<'a, str>>,
}
