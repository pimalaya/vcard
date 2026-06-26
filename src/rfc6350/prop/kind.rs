//! The KIND property (RFC 6350 section 6.1.4).

use alloc::borrow::Cow;

/// The KIND property name.
pub const KIND: &str = "KIND";

/// The kind of entity a card represents (the KIND property).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum VcardKind<'a> {
    /// A single person; the default.
    #[default]
    Individual,
    /// A group of entities.
    Group,
    /// An organisation.
    Org,
    /// A named geographical place.
    Location,
    /// Any IANA or X- kind.
    Other(Cow<'a, str>),
}
