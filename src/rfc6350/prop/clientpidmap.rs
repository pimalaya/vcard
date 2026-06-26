//! The CLIENTPIDMAP property (RFC 6350 section 6.7.7).

use alloc::borrow::Cow;

/// The CLIENTPIDMAP property name.
pub const CLIENTPIDMAP: &str = "CLIENTPIDMAP";

/// A PID-to-source mapping (the CLIENTPIDMAP property).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardClientPidMap<'a> {
    /// The source identifier referenced by PID parameters.
    pub id: u32,
    /// The URI identifying that source.
    pub uri: Cow<'a, str>,
}
