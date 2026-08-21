//! # CLIENTPIDMAP value
//!
//! The decoded `CLIENTPIDMAP` value.
//!
//! `CLIENTPIDMAP` ties the PID source identifiers used by `PID` parameters to
//! the client (a URI) that produced them: a structured RFC 6350 6.7.7 value of
//! two `;`-ordered components, a small integer source id and a URI, which this
//! bespoke type names.

use alloc::borrow::Cow;

/// The decoded CLIENTPIDMAP value: a source id and the URI that produced it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardClientPidMap<'a> {
    /// The PID source identifier (a small positive integer).
    pub id: Cow<'a, str>,
    /// The URI of the client that produced the identifiers.
    pub uri: Cow<'a, str>,
}
