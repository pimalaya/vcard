//! # Decode (syntax to model)
//!
//! The read side of the bridge: project a raw syntax tree onto the decoded
//! model.
//!
//! `unescape` is the read codec (it resolves the RFC 6350 value escapes). On top
//! of it sit the `decode` methods, one per syntactic node: a
//! [`VcardValueNode`] decodes its components, a [`VcardParamNode`] decodes into a
//! [`VcardParam`], a [`VcardLine`] decodes into a [`VcardProp`], and a
//! [`VcardCst`] decodes into a whole [`Vcard`]. The name dispatch (which value
//! kind a property or parameter decodes to) is the match in
//! [`VcardLine::decode`] and [`VcardParamNode::decode`], kept here so the generic
//! nodes and the decoded model both stay free of it. Each decoded value type
//! also gets an inherent `decode` that the per-name lens markers delegate to.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::v40::{
    param::VcardParam,
    prop::VcardProp,
    tree::{cst::VcardCst, line::VcardLine, param::VcardParamNode, value::VcardValueNode},
    value::{
        VcardUnknownValue, VcardValue,
        adr::VcardAdr,
        client_pid_map::VcardClientPidMap,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        gender::VcardGender,
        language::VcardLanguageTag,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
    },
    vcard::Vcard,
    version::VcardVersion,
};

impl VcardCst<'_> {
    /// Decode the whole card into the semantic [`Vcard`] model.
    pub fn decode(&self) -> Vcard<'_> {
        Vcard {
            version: VcardVersion::from(Cow::Borrowed(self.version.raw_value())),
            properties: self.props.iter().map(VcardLine::decode).collect(),
        }
    }
}

impl VcardLine<'_> {
    /// Decode the line into a typed property, dispatching the value on the name.
    pub fn decode(&self) -> VcardProp<'_> {
        let name = self.name.get();
        let params = self.params.iter().map(VcardParamNode::decode).collect();
        let node = &self.value;

        let value = {
            use crate::v40::prop::*;

            match name.to_ascii_uppercase().as_str() {
                VCARD_FN | VCARD_KIND | VCARD_XML | VCARD_TEL | VCARD_EMAIL | VCARD_TITLE
                | VCARD_ROLE | VCARD_NOTE | VCARD_PRODID | VCARD_TZ => {
                    VcardValue::Text(VcardText::decode(node))
                }
                VCARD_NICKNAME | VCARD_CATEGORIES => {
                    VcardValue::TextList(VcardTextList::decode(node))
                }
                VCARD_SOURCE | VCARD_PHOTO | VCARD_IMPP | VCARD_LOGO | VCARD_MEMBER
                | VCARD_RELATED | VCARD_SOUND | VCARD_UID | VCARD_GEO | VCARD_URL | VCARD_KEY
                | VCARD_FBURL | VCARD_CALADRURI | VCARD_CALURI => {
                    VcardValue::Uri(VcardUri::decode(node))
                }
                VCARD_BDAY | VCARD_ANNIVERSARY => {
                    VcardValue::DateAndOrTime(VcardDateAndOrTime::decode(node))
                }
                VCARD_REV => VcardValue::Timestamp(VcardTimestamp::decode(node)),
                VCARD_LANG => VcardValue::LanguageTag(VcardLanguageTag::decode(node)),
                VCARD_N => VcardValue::N(VcardN::decode(node)),
                VCARD_ADR => VcardValue::Adr(VcardAdr::decode(node)),
                VCARD_GENDER => VcardValue::Gender(VcardGender::decode(node)),
                VCARD_ORG => VcardValue::Org(VcardOrg::decode(node)),
                VCARD_CLIENTPIDMAP => VcardValue::ClientPidMap(VcardClientPidMap::decode(node)),

                _ => VcardValue::Unknown(VcardUnknownValue {
                    components: node
                        .components
                        .iter()
                        .map(|component| component.iter().map(|v| unescape(v.get())).collect())
                        .collect(),
                }),
            }
        };

        VcardProp {
            name: Cow::Borrowed(name),
            params,
            value,
        }
    }
}

impl VcardParamNode<'_> {
    /// Decode the parameter into a typed parameter, dispatching on the name.
    pub fn decode(&self) -> VcardParam<'_> {
        use crate::v40::param::*;

        match self.name.get().to_ascii_uppercase().as_str() {
            VCARD_LANGUAGE => VcardParam::Language(self.scalar()),
            VCARD_VALUE => VcardParam::Value(self.scalar()),
            VCARD_PREF => VcardParam::Pref(self.scalar()),
            VCARD_ALTID => VcardParam::AltId(self.scalar()),
            VCARD_PID => VcardParam::Pid(self.list()),
            VCARD_TYPE => VcardParam::Type(self.list()),
            VCARD_MEDIATYPE => VcardParam::MediaType(self.scalar()),
            VCARD_CALSCALE => VcardParam::CalScale(self.scalar()),
            VCARD_SORT_AS => VcardParam::SortAs(self.list()),
            VCARD_GEO => VcardParam::Geo(self.scalar()),
            VCARD_TZ => VcardParam::Tz(self.scalar()),
            VCARD_LABEL => VcardParam::Label(self.scalar()),

            _ => VcardParam::Unknown {
                name: unescape(self.name.get()),
                values: self.list(),
            },
        }
    }

    /// The parameter's first value, decoded (empty when there is none).
    fn scalar(&self) -> Cow<'_, str> {
        self.values
            .first()
            .map(|v| unescape(v.get()))
            .unwrap_or(Cow::Borrowed(""))
    }

    /// The parameter's values, decoded.
    fn list(&self) -> Vec<Cow<'_, str>> {
        self.values.iter().map(|v| unescape(v.get())).collect()
    }
}

impl VcardValueNode<'_> {
    /// Decode the `i`th component into a clean (unescaped) value list.
    pub fn decode_at(&self, i: usize) -> Vec<Cow<'_, str>> {
        self.components
            .get(i)
            .map(|leaves| leaves.iter().map(|leaf| unescape(leaf.get())).collect())
            .unwrap_or_default()
    }

    /// Decode the `i`th component's first value (empty when there is none).
    pub fn decode_scalar_at(&self, i: usize) -> Cow<'_, str> {
        self.components
            .get(i)
            .and_then(|leaves| leaves.first())
            .map(|leaf| unescape(leaf.get()))
            .unwrap_or(Cow::Borrowed(""))
    }
}

impl VcardText<'_> {
    /// Decode a single text value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardText<'v> {
        VcardText(node.decode_scalar_at(0))
    }
}

impl VcardTextList<'_> {
    /// Decode a comma-separated text list from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardTextList<'v> {
        VcardTextList(node.decode_at(0))
    }
}

impl VcardUri<'_> {
    /// Decode a URI value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardUri<'v> {
        VcardUri(node.decode_scalar_at(0))
    }
}

impl VcardDateAndOrTime<'_> {
    /// Decode a date-and-or-time value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardDateAndOrTime<'v> {
        VcardDateAndOrTime(node.decode_scalar_at(0))
    }
}

impl VcardTimestamp<'_> {
    /// Decode a timestamp value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardTimestamp<'v> {
        VcardTimestamp(node.decode_scalar_at(0))
    }
}

impl VcardLanguageTag<'_> {
    /// Decode a language-tag value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardLanguageTag<'v> {
        VcardLanguageTag(node.decode_scalar_at(0))
    }
}

impl VcardN<'_> {
    /// Decode the structured N value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardN<'v> {
        VcardN {
            family: node.decode_at(0),
            given: node.decode_at(1),
            additional: node.decode_at(2),
            prefixes: node.decode_at(3),
            suffixes: node.decode_at(4),
        }
    }
}

impl VcardAdr<'_> {
    /// Decode the structured ADR value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardAdr<'v> {
        VcardAdr {
            po_box: node.decode_at(0),
            extended: node.decode_at(1),
            street: node.decode_at(2),
            locality: node.decode_at(3),
            region: node.decode_at(4),
            postal_code: node.decode_at(5),
            country: node.decode_at(6),
        }
    }
}

impl VcardGender<'_> {
    /// Decode the structured GENDER value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardGender<'v> {
        VcardGender {
            sex: node.decode_scalar_at(0),
            identity: node.decode_scalar_at(1),
        }
    }
}

impl VcardOrg<'_> {
    /// Decode the structured ORG value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardOrg<'v> {
        let units = (0..node.components.len())
            .map(|i| node.decode_scalar_at(i))
            .collect();
        VcardOrg(units)
    }
}

impl VcardClientPidMap<'_> {
    /// Decode the structured CLIENTPIDMAP value from a syntax node.
    pub fn decode<'v>(node: &'v VcardValueNode<'_>) -> VcardClientPidMap<'v> {
        VcardClientPidMap {
            id: node.decode_scalar_at(0),
            uri: node.decode_scalar_at(1),
        }
    }
}

/// Resolve the RFC 6350 value escapes `\\` `\,` `\;` `\n`, borrowing when there
/// is nothing to unescape.
pub(crate) fn unescape(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec};

    use crate::v40::{
        tree::{decode::unescape, value::VcardValueNode},
        value::n::VcardN,
    };

    #[test]
    fn unescapes_value_escapes_and_borrows_when_clean() {
        assert_eq!(unescape(r"a\,b\;c\nd"), "a,b;c\nd");
        assert!(matches!(unescape("plain"), Cow::Borrowed("plain")));
    }

    #[test]
    fn decodes_components_into_scalar_and_list() {
        let node = VcardValueNode::parse("a,b;c");
        assert_eq!(
            node.decode_at(0),
            vec![Cow::Borrowed("a"), Cow::Borrowed("b")],
        );
        assert_eq!(node.decode_scalar_at(1), Cow::Borrowed("c"));
        assert_eq!(node.decode_scalar_at(9), Cow::Borrowed(""));
    }

    #[test]
    fn decodes_the_structured_n_value() {
        let node = VcardValueNode::parse("Doe;John;;Dr.;");
        let n = VcardN::decode(&node);
        assert_eq!(n.family, vec![Cow::Borrowed("Doe")]);
        assert_eq!(n.given, vec![Cow::Borrowed("John")]);
        assert_eq!(n.suffixes, vec![Cow::Borrowed("")]);
    }
}
