//! # ADR value
//!
//! The decoded `ADR` (structured address) value.
//!
//! `ADR` is a structured RFC 6350 value of seven `;`-ordered components (post
//! office box, extended address, street, locality, region, postal code,
//! country), each a possibly multi-valued `,`-separated list. The first two are
//! deprecated by the RFC but kept for round-tripping. This bespoke type names
//! the components so callers read `adr.street` rather than indexing. Pure,
//! always-unescaped data; framing and escaping live on the syntax side
//! ([`crate::v4_0::tree`]). The wire name lives on [`crate::v4_0::prop::VcardProp::name`].

use alloc::{borrow::Cow, vec::Vec};

/// The decoded ADR value: seven address components, each a clean (unescaped)
/// value list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardAdr<'a> {
    /// Post office box (deprecated, kept for round-tripping).
    pub po_box: Vec<Cow<'a, str>>,
    /// Extended address (deprecated, kept for round-tripping).
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
