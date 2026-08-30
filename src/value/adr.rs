//! # ADR value
//!
//! The decoded `ADR` (structured address) value.
//!
//! `ADR` is a structured value of `;`-ordered components, each a possibly
//! multi-valued `,`-separated list: seven in RFC 6350 6.3.1 (post office box,
//! extended address, street, locality, region, postal code, country; the first
//! two deprecated but kept for round-tripping).
//!
//! RFC 9554 extends them to eighteen (room, apartment, floor, street number,
//! street name, building, block, subdistrict, district, landmark, direction).
//! This bespoke type names them, so callers read `adr.street` rather than
//! indexing.

use alloc::{borrow::Cow, vec::Vec};

/// The decoded ADR value: the eighteen address components, each a clean
/// (unescaped) value list.
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
    /// Room, suite number or identifier (RFC 9554).
    pub room: Vec<Cow<'a, str>>,
    /// Extension designation such as an apartment number (RFC 9554).
    pub apartment: Vec<Cow<'a, str>>,
    /// Floor or level (RFC 9554).
    pub floor: Vec<Cow<'a, str>>,
    /// Street number (RFC 9554).
    pub street_number: Vec<Cow<'a, str>>,
    /// Street name (RFC 9554).
    pub street_name: Vec<Cow<'a, str>>,
    /// Building or building part (RFC 9554).
    pub building: Vec<Cow<'a, str>>,
    /// Block name or number (RFC 9554).
    pub block: Vec<Cow<'a, str>>,
    /// Subdistrict (RFC 9554).
    pub subdistrict: Vec<Cow<'a, str>>,
    /// District (RFC 9554).
    pub district: Vec<Cow<'a, str>>,
    /// Landmark (RFC 9554).
    pub landmark: Vec<Cow<'a, str>>,
    /// Cardinal direction (RFC 9554).
    pub direction: Vec<Cow<'a, str>>,
}

impl VcardAdr<'_> {
    /// Whether any RFC 9554 component (room through direction) carries a
    /// value, which is what tells a seven-component wire value from an
    /// eighteen-component one.
    pub fn has_extended_components(&self) -> bool {
        [
            &self.room,
            &self.apartment,
            &self.floor,
            &self.street_number,
            &self.street_name,
            &self.building,
            &self.block,
            &self.subdistrict,
            &self.district,
            &self.landmark,
            &self.direction,
        ]
        .into_iter()
        .any(|component| component.iter().any(|value| !value.is_empty()))
    }
}
