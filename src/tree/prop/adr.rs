//! # ADR lens
//!
//! The `ADR` (structured address) property lens, with a cursor naming its seven
//! components. Like [`NCursor`](crate::tree::prop::n::NCursor), getters decode
//! and setters encode in place, leaving the other components (and every
//! parameter) byte for byte intact.
//!
//! See RFC 6350 6.3.1.

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        line::VcardLine,
        param::VcardParamLens,
        prop::{VcardPropLens, VcardPropSpec},
        value::VcardValueNode,
    },
    value::{VcardValueKind, adr::VcardAdr},
    version::VcardVersion,
};

/// The `ADR` property lens.
pub struct ADR;

impl VcardPropLens for ADR {
    type Target<'v> = VcardAdr<'v>;

    type Cursor<'c, 'a>
        = AdrCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardAdr<'v> {
        VcardAdr::decode(value)
    }

    fn encode(decoded: &VcardAdr<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> AdrCursor<'c, 'a> {
        AdrCursor { line }
    }
}

impl VcardPropSpec for ADR {
    const PROP: VcardPropKind = VcardPropKind::Adr;

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Adr]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Label,
            VcardParamKind::Language,
            VcardParamKind::Geo,
            VcardParamKind::Tz,
            VcardParamKind::AltId,
            VcardParamKind::Pid,
            VcardParamKind::Pref,
            VcardParamKind::Type,
            VcardParamKind::Value,
        ]
    }
}

/// A typed cursor over an ADR line, naming its seven components.
pub struct AdrCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl AdrCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardAdr<'_> {
        VcardAdr::decode(&self.line.value)
    }

    /// The post office box (deprecated), decoded.
    pub fn po_box(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(0)
    }

    /// Set the post office box.
    pub fn set_po_box<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(0, values);
    }

    /// The extended address (deprecated), decoded.
    pub fn extended(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(1)
    }

    /// Set the extended address.
    pub fn set_extended<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(1, values);
    }

    /// The street address, decoded.
    pub fn street(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(2)
    }

    /// Set the street address.
    pub fn set_street<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(2, values);
    }

    /// The locality (city), decoded.
    pub fn locality(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(3)
    }

    /// Set the locality.
    pub fn set_locality<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(3, values);
    }

    /// The region (state or province), decoded.
    pub fn region(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(4)
    }

    /// Set the region.
    pub fn set_region<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(4, values);
    }

    /// The postal code, decoded.
    pub fn postal_code(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(5)
    }

    /// Set the postal code.
    pub fn set_postal_code<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(5, values);
    }

    /// The country name, decoded.
    pub fn country(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(6)
    }

    /// Set the country name.
    pub fn set_country<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(6, values);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}
