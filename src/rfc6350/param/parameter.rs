//! A property parameter (RFC 6350 section 5).

use alloc::{borrow::Cow, vec::Vec};

/// One parameter: its name and its comma-separated values, as written.
///
/// A property's parameters are an ordered `Vec<VcardParameter>` in which any
/// name may appear (and repeat), exactly as the wire carries them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardParameter<'a> {
    /// The parameter name, for example TYPE or PID.
    pub name: Cow<'a, str>,
    /// The values, one per comma-separated entry; empty when valueless.
    pub values: Vec<Cow<'a, str>>,
}
