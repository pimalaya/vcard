//! The vCard aggregate.

use alloc::{borrow::Cow, vec::Vec};

use crate::rfc6350::version::VcardVersion;

/// The BEGIN delimiter name, opening a card.
pub const BEGIN: &str = "BEGIN";
/// The END delimiter name, closing a card.
pub const END: &str = "END";

/// The value carried by the BEGIN and END delimiters.
pub const VCARD: &str = "VCARD";

/// A whole vCard: its version and its properties, in source order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Vcard<'a> {
    /// The card version.
    pub version: VcardVersion,
    /// Every property, kept in the order it appears.
    pub properties: Vec<VcardProperty<'a>>,
}

impl<'a> Vcard<'a> {
    /// The properties carrying the given name, matched case-insensitively.
    pub fn get(&self, name: &str) -> impl Iterator<Item = &VcardProperty<'a>> {
        self.properties
            .iter()
            .filter(move |property| property.name.eq_ignore_ascii_case(name))
    }
}

/// One property line: its name, raw parameters and raw value, as written.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardProperty<'a> {
    /// The property name, with any group prefix, for example FN or item1.TEL.
    pub name: Cow<'a, str>,
    /// The raw parameters, leading with `;` when present, empty otherwise.
    pub params: Cow<'a, str>,
    /// The raw value, escaping intact.
    pub value: Cow<'a, str>,
}
