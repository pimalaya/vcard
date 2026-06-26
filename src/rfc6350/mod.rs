//! The vCard model (RFC 6350): the shared aggregate, version, property and
//! value types at the top, with property names and parameter names split into
//! their own namespaces because the two name sets overlap (GEO, TZ).

pub mod extension;
pub mod param;
pub mod prop;
pub mod value;
pub mod version;

pub mod vcard;
