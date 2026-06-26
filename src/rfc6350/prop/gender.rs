//! The GENDER property (RFC 6350 section 6.2.7).

use alloc::borrow::Cow;

/// The GENDER property name.
pub const GENDER: &str = "GENDER";

/// The gender value (the GENDER property): a sex plus a free-form identity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardGender<'a> {
    /// The sex component, if given.
    pub sex: Option<VcardSex>,
    /// The free-form gender identity, if given.
    pub identity: Option<Cow<'a, str>>,
}

/// The sex component of a [`VcardGender`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcardSex {
    /// Male.
    Male,
    /// Female.
    Female,
    /// Other.
    Other,
    /// None (not applicable).
    NotApplicable,
    /// Unknown.
    Unknown,
}
