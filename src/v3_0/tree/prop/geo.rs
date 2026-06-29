//! # GEO lens
//!
//! The `GEO` (geographic position) property lens, with a cursor naming its two
//! components (latitude and longitude). Like
//! [`NCursor`](crate::v3_0::tree::prop::n::NCursor), getters decode and setters
//! encode in place, leaving the other component (and every parameter) byte for
//! byte intact.

use alloc::borrow::Cow;

use crate::v3_0::{
    prop::VCARD_GEO,
    tree::{line::VcardLine, param::VcardParamLens, prop::VcardPropLens, value::VcardValueNode},
    value::geo::VcardGeo,
};

/// The `GEO` property lens.
pub struct GEO;

impl VcardPropLens for GEO {
    const NAME: &'static str = VCARD_GEO;

    type Target<'v> = VcardGeo<'v>;

    type Cursor<'c, 'a>
        = GeoCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardGeo<'v> {
        VcardGeo::decode(value)
    }

    fn encode(decoded: &VcardGeo<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> GeoCursor<'c, 'a> {
        GeoCursor { line }
    }
}

/// A typed cursor over a GEO line, naming its latitude and longitude.
pub struct GeoCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl GeoCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardGeo<'_> {
        VcardGeo::decode(&self.line.value)
    }

    /// The latitude, decoded.
    pub fn latitude(&self) -> Cow<'_, str> {
        self.line.value.decode_scalar_at(0)
    }

    /// Set the latitude.
    pub fn set_latitude(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(0, &[value]);
    }

    /// The longitude, decoded.
    pub fn longitude(&self) -> Cow<'_, str> {
        self.line.value.decode_scalar_at(1)
    }

    /// Set the longitude.
    pub fn set_longitude(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(1, &[value]);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}
