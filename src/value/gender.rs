//! # GENDER value
//!
//! The decoded `GENDER` value.
//!
//! `GENDER` is a structured RFC 6350 6.2.7 value of two `;`-ordered components:
//! a single-letter sex code (`M`, `F`, `O`, `N`, `U`, or empty) and a free-text
//! gender identity. This bespoke type names the two parts. Pure,
//! always-unescaped data; framing and escaping live on the syntax side
//! ([`crate::tree`]). The wire name lives on [`crate::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// The decoded GENDER value: a sex code and a free-text identity.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardGender<'a> {
    /// The sex code (`M`, `F`, `O`, `N`, `U`, or empty).
    pub sex: Cow<'a, str>,
    /// The free-text gender identity.
    pub identity: Cow<'a, str>,
}
