//! # Geo value
//!
//! The decoded geographic-position value kind.
//!
//! Backs the `GEO` property (RFC 6350 6.5.2) in vCard 2.1 (a `,`-separated
//! pair) and 3.0 (a `;`-separated pair); vCard 4.0 carries `GEO` as a `geo:`
//! URI instead, decoded to [`VcardUri`](crate::value::uri::VcardUri). The
//! latitude and longitude are kept as raw text.

use alloc::borrow::Cow;

/// A decoded `GEO` value: a latitude/longitude pair, kept as raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardGeo<'a> {
    /// The latitude, as raw text.
    pub latitude: Cow<'a, str>,
    /// The longitude, as raw text.
    pub longitude: Cow<'a, str>,
}
