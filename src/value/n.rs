//! # N value
//!
//! The decoded `N` (structured name) value.
//!
//! `N` is one of the few genuinely structured RFC 6350 values (section 6.2.2):
//! five `;`-ordered components (family, given, additional, prefixes, suffixes),
//! each a possibly multi-valued `,`-separated list. This bespoke type names
//! them, so callers read `name.family` rather than indexing a raw component
//! vector.

use alloc::{borrow::Cow, vec::Vec};

/// The decoded N value: five name components, each a clean (unescaped) value
/// list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardN<'a> {
    /// Family names.
    pub family: Vec<Cow<'a, str>>,
    /// Given names.
    pub given: Vec<Cow<'a, str>>,
    /// Additional names.
    pub additional: Vec<Cow<'a, str>>,
    /// Honorific prefixes.
    pub prefixes: Vec<Cow<'a, str>>,
    /// Honorific suffixes.
    pub suffixes: Vec<Cow<'a, str>>,
}
