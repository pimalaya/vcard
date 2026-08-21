//! # ADR lens
//!
//! The `ADR` (structured address) property lens, with a cursor naming its
//! components: the seven RFC 6350 ones and the eleven RFC 9554 extensions.
//! Like [`VcardNCursor`](crate::tree::prop::n::VcardNCursor), getters decode and
//! setters encode in place, leaving the other components byte for byte intact.
//!
//! See RFC 6350 6.3.1 and RFC 9554.

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        codec::VcardCodec,
        line::VcardLine,
        param::lens::VcardParamLens,
        prop::{lens::VcardPropLens, spec::VcardPropSpec},
    },
    value::{VcardValueKind, adr::VcardAdr},
    version::VcardVersion,
};

/// The `ADR` property lens.
pub struct ADR;

impl VcardPropLens for ADR {
    type Target<'v> = VcardAdr<'v>;

    type Cursor<'c, 'a>
        = VcardAdrCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardAdrCursor<'c, 'a> {
        VcardAdrCursor { line }
    }
}

impl VcardPropSpec for ADR {
    const KIND: VcardPropKind = VcardPropKind::Adr;

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Adr]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[
            VcardParamKind::Label,
            VcardParamKind::Language,
            VcardParamKind::Phonetic,
            VcardParamKind::Script,
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
pub struct VcardAdrCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl VcardAdrCursor<'_, '_> {
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

    /// The room (RFC 9554), decoded.
    pub fn room(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(7)
    }

    /// Set the room.
    pub fn set_room<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(7, values);
    }

    /// The apartment (RFC 9554), decoded.
    pub fn apartment(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(8)
    }

    /// Set the apartment.
    pub fn set_apartment<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(8, values);
    }

    /// The floor (RFC 9554), decoded.
    pub fn floor(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(9)
    }

    /// Set the floor.
    pub fn set_floor<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(9, values);
    }

    /// The street number (RFC 9554), decoded.
    pub fn street_number(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(10)
    }

    /// Set the street number.
    pub fn set_street_number<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(10, values);
    }

    /// The street name (RFC 9554), decoded.
    pub fn street_name(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(11)
    }

    /// Set the street name.
    pub fn set_street_name<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(11, values);
    }

    /// The building (RFC 9554), decoded.
    pub fn building(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(12)
    }

    /// Set the building.
    pub fn set_building<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(12, values);
    }

    /// The block (RFC 9554), decoded.
    pub fn block(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(13)
    }

    /// Set the block.
    pub fn set_block<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(13, values);
    }

    /// The subdistrict (RFC 9554), decoded.
    pub fn subdistrict(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(14)
    }

    /// Set the subdistrict.
    pub fn set_subdistrict<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(14, values);
    }

    /// The district (RFC 9554), decoded.
    pub fn district(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(15)
    }

    /// Set the district.
    pub fn set_district<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(15, values);
    }

    /// The landmark (RFC 9554), decoded.
    pub fn landmark(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(16)
    }

    /// Set the landmark.
    pub fn set_landmark<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(16, values);
    }

    /// The cardinal direction (RFC 9554), decoded.
    pub fn direction(&self) -> Vec<Cow<'_, str>> {
        self.line.value.decode_at(17)
    }

    /// Set the cardinal direction.
    pub fn set_direction<S: AsRef<str>>(&mut self, values: &[S]) {
        self.line.value.set_at(17, values);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}
