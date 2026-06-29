//! # GEO value
//!
//! The decoded `GEO` (geographic position) value.
//!
//! In vCard 2.1, `GEO` is a structured value of two floats separated by `;`:
//! latitude then longitude. They are kept as their raw text (no float parsing,
//! so the value round-trips exactly). Pure decoded data; the `;` splitting and
//! `\;` escaping live on the syntax side ([`crate::v2_1::tree`]). The wire name
//! lives on [`crate::v2_1::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// The decoded GEO value: latitude and longitude, as raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardGeo<'a> {
    /// Latitude, in decimal degrees.
    pub latitude: Cow<'a, str>,
    /// Longitude, in decimal degrees.
    pub longitude: Cow<'a, str>,
}
