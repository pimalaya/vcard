//! The N property (RFC 6350 section 6.2.2).

use alloc::{borrow::Cow, vec::Vec};

/// The N property name.
pub const N: &str = "N";

/// A structured name (the N property), each part possibly multi-valued.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardName<'a> {
    /// Family names (surnames).
    pub family: Vec<Cow<'a, str>>,
    /// Given names.
    pub given: Vec<Cow<'a, str>>,
    /// Additional or middle names.
    pub additional: Vec<Cow<'a, str>>,
    /// Honorific prefixes.
    pub prefixes: Vec<Cow<'a, str>>,
    /// Honorific suffixes.
    pub suffixes: Vec<Cow<'a, str>>,
}
