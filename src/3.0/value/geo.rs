//! # GEO value
//!
//! The decoded `GEO` (geographic position) value.
//!
//! In vCard 3.0, `GEO` is a structured value of two floats separated by `;`:
//! latitude then longitude. They are kept as their raw text (no float parsing,
//! so the value round-trips exactly). This bespoke type names the two parts so
//! callers read `geo.latitude` rather than indexing. Pure, always-unescaped
//! data; the `;` splitting and `\;` escaping live on the syntax side
//! ([`crate::v30::tree`]). The wire name lives on
//! [`crate::v30::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// The decoded GEO value: latitude and longitude, as raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardGeo<'a> {
    /// Latitude, in decimal degrees.
    pub latitude: Cow<'a, str>,
    /// Longitude, in decimal degrees.
    pub longitude: Cow<'a, str>,
}
