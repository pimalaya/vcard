//! Properties kept generically, by name.

use alloc::{borrow::Cow, vec, vec::Vec};

use crate::rfc6350::{
    param::{parameter::VcardParameter, value::VALUE},
    value::VcardUriOrText,
};

/// A property kept generically by name: any extension, IANA or X- property
/// a card carries that is not one of the modelled RFC 6350 properties.
///
/// Typed views in the extension RFC modules convert into this with [`From`];
/// the reverse (reading a typed value back out) is interpretation and
/// belongs to the parser, not here.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardExtension<'a> {
    /// The property name as written, for example BIRTHPLACE or X-ABLABEL.
    pub name: Cow<'a, str>,
    /// The parameters decorating it, in source order.
    pub params: Vec<VcardParameter<'a>>,
    /// The value as decoded components (separated by `;` on the wire), each
    /// a list of values (separated by `,`), free of separators and escaping.
    pub values: Vec<Vec<Cow<'a, str>>>,
}

impl<'a> VcardExtension<'a> {
    /// Build a single-value extension property.
    pub(crate) fn single(
        name: &'a str,
        params: Vec<VcardParameter<'a>>,
        value: Cow<'a, str>,
    ) -> Self {
        Self {
            name: Cow::Borrowed(name),
            params,
            values: vec![vec![value]],
        }
    }

    /// Build a single-value extension property whose value is a URI or text,
    /// recording the choice in the VALUE parameter so it round-trips
    /// whatever the property default.
    pub(crate) fn uri_or_text(
        name: &'a str,
        mut params: Vec<VcardParameter<'a>>,
        value: VcardUriOrText<'a>,
    ) -> Self {
        let (value, value_type) = match value {
            VcardUriOrText::Uri(uri) => (uri, "uri"),
            VcardUriOrText::Text(text) => (text, "text"),
        };

        params.push(VcardParameter {
            name: Cow::Borrowed(VALUE),
            values: vec![Cow::Borrowed(value_type)],
        });

        Self::single(name, params, value)
    }
}
