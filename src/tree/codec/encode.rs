//! # Encode (model to syntax)
//!
//! The write side of the structural bridge: project the decoded model onto a
//! raw syntax tree. A value's [`Codec`] impl encodes it into a
//! [`VcardValueNode`], a [`VcardParam`] encodes into a [`VcardParamNode`], a
//! [`VcardProp`] encodes into a [`VcardLine`] (its name taken verbatim from the
//! property, its value delegated to the value codec), and a [`Vcard`] encodes
//! into a whole [`VcardCst`]. The whole card is encoded for its version's
//! [`Escaper`], which the value codecs use to escape every leaf and to pick any
//! version-specific value shape; byte-preserving edits are the cursors' job,
//! not this module's. [`Display`](core::fmt::Display) for [`Vcard`] renders a
//! decoded card straight to its serialized bytes through here. Value leaves are
//! escaped by the sibling [`escape`](crate::tree::codec::escape) codec.

use core::fmt;

use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

use crate::{
    param::VcardParam,
    prop::VcardProp,
    tree::{
        codec::{Codec, escape::escape_with, mode::Escaper},
        cst::VcardCst,
        leaf::{VcardLeaf, VcardValueLeaf},
        line::VcardLine,
        param::VcardParamNode,
        value::VcardValueNode,
    },
    vcard::Vcard,
};

impl Vcard<'_> {
    /// Encode the whole card into a CST for its version's escaping mode.
    pub fn encode(&self) -> VcardCst<'static> {
        let escaper = Escaper::for_version(self.version);

        let mut cst = VcardCst::v4();
        // NOTE: v4() seeds a VERSION line as the first property; set it to this
        // card's version, then append the rest. VERSION stays an ordinary
        // property.
        cst.props[0] = VcardLine::text("VERSION", self.version.to_string());
        cst.props
            .extend(self.properties.iter().map(|prop| prop.encode(escaper)));

        cst
    }
}

impl<'a> From<Vcard<'a>> for VcardCst<'static> {
    fn from(card: Vcard<'a>) -> Self {
        card.encode()
    }
}

impl VcardProp<'_> {
    /// Encode the property into a raw content line for the given escaping mode,
    /// dispatching on its value.
    pub fn encode(&self, escaper: Escaper) -> VcardLine<'static> {
        VcardLine {
            name: VcardLeaf::from(self.name.to_string()),
            params: self.params.iter().map(VcardParam::encode).collect(),
            value: self.value.encode(escaper),
            eol: VcardLeaf::from("\r\n".to_string()),
        }
    }
}

impl VcardParam<'_> {
    /// Encode the parameter into a raw parameter node, dispatching on its kind.
    pub fn encode(&self) -> VcardParamNode<'static> {
        use crate::param::VcardParamKind::*;

        match self {
            VcardParam::Language(v) => param_scalar(&Language, v),
            VcardParam::Charset(v) => param_scalar(&Charset, v),
            VcardParam::Encoding(v) => param_scalar(&Encoding, v),
            VcardParam::Value(v) => param_scalar(&Value, v),
            VcardParam::Pref(v) => param_scalar(&Pref, v),
            VcardParam::AltId(v) => param_scalar(&AltId, v),
            VcardParam::Pid(vs) => param_list(&Pid, vs),
            VcardParam::Type(vs) => param_list(&Type, vs),
            VcardParam::MediaType(v) => param_scalar(&MediaType, v),
            VcardParam::CalScale(v) => param_scalar(&CalScale, v),
            VcardParam::SortAs(vs) => param_list(&SortAs, vs),
            VcardParam::Geo(v) => param_scalar(&Geo, v),
            VcardParam::Tz(v) => param_scalar(&Tz, v),
            VcardParam::Label(v) => param_scalar(&Label, v),

            VcardParam::Unknown { name, values } => VcardParamNode {
                name: VcardLeaf::from(name.to_string()),
                values: values
                    .iter()
                    .map(|v| VcardLeaf::from(v.to_string()))
                    .collect(),
            },
        }
    }
}

/// Serialize the decoded card by encoding it into a CST (canonical).
impl fmt::Display for Vcard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

/// A one-component, one-value syntax node, escaping the value by the given
/// mode.
pub(crate) fn scalar_node(value: &str, escaper: Escaper) -> VcardValueNode<'static> {
    VcardValueNode::from_components(vec![encode_component(&[value], escaper)], escaper)
}

/// Escape and own a clean value list into one component, by escaping mode.
pub(crate) fn encode_component<S: AsRef<str>>(
    values: &[S],
    escaper: Escaper,
) -> Vec<VcardValueLeaf<'static>> {
    values
        .iter()
        .map(|v| VcardValueLeaf::from(escape_with(v.as_ref().as_bytes(), escaper).into_owned()))
        .collect()
}

/// A parameter node from a single value (parameter values are not escaped: the
/// wire form is quoted, not backslash-escaped).
fn param_scalar(name: &str, value: &str) -> VcardParamNode<'static> {
    VcardParamNode {
        name: VcardLeaf::from(name.to_string()),
        values: vec![VcardLeaf::from(value.to_string())],
    }
}

/// A parameter node from a value list (parameter values are not escaped).
fn param_list(name: &str, values: &[Cow<'_, str>]) -> VcardParamNode<'static> {
    VcardParamNode {
        name: VcardLeaf::from(name.to_string()),
        values: values
            .iter()
            .map(|v| VcardLeaf::from(v.to_string()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::{
        tree::{
            codec::{Codec, mode::Escaper},
            cst::VcardCst,
        },
        value::{n::VcardN, text::VcardText},
    };

    #[test]
    fn encodes_a_text_value_escaping_it() {
        let node = VcardText(Cow::Borrowed("hi, there")).encode(Escaper::Modern);
        assert_eq!(node.to_string(), r"hi\, there");
    }

    #[test]
    fn encodes_the_structured_n_value_with_all_components() {
        let n = VcardN {
            family: vec![Cow::Borrowed("Doe")],
            ..Default::default()
        };
        assert_eq!(n.encode(Escaper::Modern).to_string(), "Doe;;;;");
    }

    #[test]
    fn encodes_the_geo_pair_in_the_cards_own_version() {
        // NOTE: A 2.1 GEO decodes to a coordinate pair; re-encoding for 2.1
        // must write it back as a comma pair, not the 3.0 semicolon form.
        let cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nGEO:37.0,-122.0\r\nEND:VCARD\r\n")
            .unwrap();
        let card = cst.decode();
        assert!(
            card.to_string().contains("GEO:37.0,-122.0\r\n"),
            "{}",
            card.to_string(),
        );

        // NOTE: The same pair round-trips through 3.0 with a semicolon.
        let cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:3.0\r\nGEO:37.0;-122.0\r\nEND:VCARD\r\n")
            .unwrap();
        let card = cst.decode();
        assert!(
            card.to_string().contains("GEO:37.0;-122.0\r\n"),
            "{}",
            card.to_string(),
        );
    }
}
