//! # ADR lens
//!
//! The `ADR` (structured address) property lens, with a cursor naming its seven
//! single-valued components. Like [`NCursor`](crate::v21::tree::prop::n::NCursor),
//! getters decode and setters encode in place, leaving the other components (and
//! every parameter) byte for byte intact.

use alloc::borrow::Cow;

use crate::v21::{
    prop::VCARD_ADR,
    tree::{line::VcardLine, param::VcardParamLens, prop::VcardPropLens, value::VcardValueNode},
    value::adr::VcardAdr,
};

/// The `ADR` property lens.
pub struct ADR;

impl VcardPropLens for ADR {
    const NAME: &'static str = VCARD_ADR;

    type Target<'v> = VcardAdr<'v>;

    type Cursor<'c, 'a>
        = AdrCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>) -> VcardAdr<'v> {
        VcardAdr::decode(line)
    }

    fn encode(decoded: &VcardAdr<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> AdrCursor<'c, 'a> {
        AdrCursor { line }
    }
}

/// A typed cursor over an ADR line, naming its seven single-valued components.
pub struct AdrCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl AdrCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardAdr<'_> {
        VcardAdr::decode(self.line)
    }

    /// The post office box, decoded.
    pub fn po_box(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().next().unwrap_or_default()
    }

    /// Set the post office box.
    pub fn set_po_box(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(0, value.as_ref());
    }

    /// The extended address, decoded.
    pub fn extended(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(1).unwrap_or_default()
    }

    /// Set the extended address.
    pub fn set_extended(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(1, value.as_ref());
    }

    /// The street address, decoded.
    pub fn street(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(2).unwrap_or_default()
    }

    /// Set the street address.
    pub fn set_street(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(2, value.as_ref());
    }

    /// The locality (city), decoded.
    pub fn locality(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(3).unwrap_or_default()
    }

    /// Set the locality.
    pub fn set_locality(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(3, value.as_ref());
    }

    /// The region (state or province), decoded.
    pub fn region(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(4).unwrap_or_default()
    }

    /// Set the region.
    pub fn set_region(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(4, value.as_ref());
    }

    /// The postal code, decoded.
    pub fn postal_code(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(5).unwrap_or_default()
    }

    /// Set the postal code.
    pub fn set_postal_code(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(5, value.as_ref());
    }

    /// The country name, decoded.
    pub fn country(&self) -> Cow<'_, str> {
        self.line.cooked().into_iter().nth(6).unwrap_or_default()
    }

    /// Set the country name.
    pub fn set_country(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(6, value.as_ref());
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}
