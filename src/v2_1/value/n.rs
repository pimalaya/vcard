//! # N value
//!
//! The decoded `N` (structured name) value.
//!
//! In vCard 2.1, `N` is five `;`-ordered components (family, given, additional,
//! prefixes, suffixes), each a **single value** (2.1 does not comma-split a
//! component into a list). This bespoke type names them so callers read
//! `name.family` rather than indexing. Pure decoded data; the `;` splitting and
//! `\;` escaping live on the syntax side ([`crate::v2_1::tree`]). The wire name
//! lives on [`crate::v2_1::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// The decoded N value: five single-value name components.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardN<'a> {
    /// Family name.
    pub family: Cow<'a, str>,
    /// Given name.
    pub given: Cow<'a, str>,
    /// Additional name.
    pub additional: Cow<'a, str>,
    /// Honorific prefix.
    pub prefix: Cow<'a, str>,
    /// Honorific suffix.
    pub suffix: Cow<'a, str>,
}
