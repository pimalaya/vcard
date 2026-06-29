//! # ADR value
//!
//! The decoded `ADR` (structured address) value.
//!
//! In vCard 2.1, `ADR` is seven `;`-ordered components (post office box,
//! extended address, street, locality, region, postal code, country), each a
//! **single value** (2.1 does not comma-split a component into a list). This
//! bespoke type names them so callers read `adr.street` rather than indexing.
//! Pure decoded data; the `;` splitting and `\;` escaping live on the syntax
//! side ([`crate::v21::tree`]). The wire name lives on
//! [`crate::v21::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// The decoded ADR value: seven single-value address components.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardAdr<'a> {
    /// Post office box.
    pub po_box: Cow<'a, str>,
    /// Extended address.
    pub extended: Cow<'a, str>,
    /// Street address.
    pub street: Cow<'a, str>,
    /// Locality (city).
    pub locality: Cow<'a, str>,
    /// Region (state or province).
    pub region: Cow<'a, str>,
    /// Postal code.
    pub postal_code: Cow<'a, str>,
    /// Country name.
    pub country: Cow<'a, str>,
}
