//! # CLIENTPIDMAP value
//!
//! The decoded `CLIENTPIDMAP` value.
//!
//! `CLIENTPIDMAP` ties the PID source identifiers used by `PID` parameters to
//! the client (a URI) that produced them. It is a structured RFC 6350 value of
//! two `;`-ordered components: a small integer source id and a URI. This bespoke
//! type names the two parts. Pure, always-unescaped data; framing and escaping
//! live on the syntax side ([`crate::v40::tree`]). The wire name lives on
//! [`crate::v40::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// The decoded CLIENTPIDMAP value: a source id and the URI that produced it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardClientPidMap<'a> {
    /// The PID source identifier (a small positive integer).
    pub id: Cow<'a, str>,
    /// The URI of the client that produced the identifiers.
    pub uri: Cow<'a, str>,
}
