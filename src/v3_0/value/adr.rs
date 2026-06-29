//! # ADR value
//!
//! The decoded `ADR` (structured address) value.
//!
//! `ADR` is a structured RFC 2426 value of seven `;`-ordered components (post
//! office box, extended address, street, locality, region, postal code,
//! country), each a possibly multi-valued `,`-separated list. This bespoke type
//! names
//! the components so callers read `adr.street` rather than indexing. Pure,
//! always-unescaped data; framing and escaping live on the syntax side
//! ([`crate::v3_0::tree`]). The wire name lives on [`crate::v3_0::prop::VcardProp::name`].

use alloc::{borrow::Cow, vec::Vec};

/// The decoded ADR value: seven address components, each a clean (unescaped)
/// value list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardAdr<'a> {
    /// Post office box.
    pub po_box: Vec<Cow<'a, str>>,
    /// Extended address.
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
