//! # Encode (model to syntax)
//!
//! The write side of the bridge: project the decoded model onto a raw syntax
//! tree.
//!
//! `escape` is the write codec (it applies the RFC 2426 value escapes). On top
//! of it sit the `encode` methods, one per decoded type: each value type encodes
//! into a [`VcardValueNode`], a [`VcardParam`] encodes into a [`VcardParamNode`],
//! a [`VcardProp`] encodes into a [`VcardLine`] (its name taken verbatim from the
//! property, its value dispatched on the value kind), and a [`Vcard`] encodes
//! into a whole [`VcardCst`]. Encoding is always canonical; byte-preserving edits
//! are the cursors' job, not this module's. [`Display`](core::fmt::Display) for
//! [`Vcard`] renders a
//! decoded card straight to its serialized bytes through here.

use core::fmt;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::tree::leaf::VcardLeaf;
use crate::v30::{
    param::VcardParam,
    prop::VcardProp,
    tree::{cst::VcardCst, line::VcardLine, param::VcardParamNode, value::VcardValueNode},
    value::{
        VcardUnknownValue, VcardValue,
        adr::VcardAdr,
        binary::VcardBinary,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        geo::VcardGeo,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
        utc_offset::VcardUtcOffset,
    },
    vcard::Vcard,
};
use crate::version::VCARD_VERSION;

impl Vcard<'_> {
    /// Encode the whole card into a canonical CST.
    pub fn encode(&self) -> VcardCst<'static> {
        let mut cst = VcardCst::v3();
        cst.version = VcardLine::text(VCARD_VERSION, self.version.as_str().to_string());
        cst.props = self.properties.iter().map(VcardProp::encode).collect();
        cst
    }
}

impl VcardProp<'_> {
    /// Encode the property into a raw content line, dispatching on its value.
    pub fn encode(&self) -> VcardLine<'static> {
        let params = self.params.iter().map(VcardParam::encode).collect();

        let value = match &self.value {
            VcardValue::Text(v) => v.encode(),
            VcardValue::TextList(v) => v.encode(),
            VcardValue::Uri(v) => v.encode(),
            VcardValue::Binary(v) => v.encode(),
            VcardValue::DateAndOrTime(v) => v.encode(),
            VcardValue::Timestamp(v) => v.encode(),
            VcardValue::UtcOffset(v) => v.encode(),
            VcardValue::N(v) => v.encode(),
            VcardValue::Adr(v) => v.encode(),
            VcardValue::Org(v) => v.encode(),
            VcardValue::Geo(v) => v.encode(),
            VcardValue::Unknown(v) => v.encode(),
        };

        VcardLine {
            name: VcardLeaf::from(self.name.to_string()),
            params,
            value,
            eol: VcardLeaf::from("\r\n".to_string()),
        }
    }
}

impl VcardParam<'_> {
    /// Encode the parameter into a raw parameter node, dispatching on its kind.
    pub fn encode(&self) -> VcardParamNode<'static> {
        use crate::v30::param::*;

        match self {
            VcardParam::Encoding(v) => param_scalar(VCARD_ENCODING, v),
            VcardParam::Language(v) => param_scalar(VCARD_LANGUAGE, v),
            VcardParam::Type(vs) => param_list(VCARD_TYPE, vs),
            VcardParam::Value(v) => param_scalar(VCARD_VALUE, v),

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

impl VcardValueNode<'_> {
    /// Set the `i`th component, escaping each value. Pads with empty components
    /// when needed; every other component is left untouched, so a parsed card
    /// keeps its bytes.
    pub fn set_at<S: AsRef<str>>(&mut self, i: usize, values: &[S]) {
        while self.components.len() <= i {
            self.components.push(Vec::new());
        }

        self.components[i] = encode_component(values);
    }
}

impl VcardText<'_> {
    /// Encode a single text value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        scalar_node(&self.0)
    }
}

impl VcardTextList<'_> {
    /// Encode a comma-separated text list into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![encode_component(&self.0)],
        }
    }
}

impl VcardUri<'_> {
    /// Encode a URI value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        scalar_node(&self.0)
    }
}

impl VcardBinary<'_> {
    /// Encode a binary value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        scalar_node(&self.0)
    }
}

impl VcardDateAndOrTime<'_> {
    /// Encode a date-and-or-time value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        scalar_node(&self.0)
    }
}

impl VcardTimestamp<'_> {
    /// Encode a timestamp value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        scalar_node(&self.0)
    }
}

impl VcardUtcOffset<'_> {
    /// Encode a UTC-offset value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        scalar_node(&self.0)
    }
}

impl VcardN<'_> {
    /// Encode the structured N value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![
                encode_component(&self.family),
                encode_component(&self.given),
                encode_component(&self.additional),
                encode_component(&self.prefixes),
                encode_component(&self.suffixes),
            ],
        }
    }
}

impl VcardAdr<'_> {
    /// Encode the structured ADR value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![
                encode_component(&self.po_box),
                encode_component(&self.extended),
                encode_component(&self.street),
                encode_component(&self.locality),
                encode_component(&self.region),
                encode_component(&self.postal_code),
                encode_component(&self.country),
            ],
        }
    }
}

impl VcardOrg<'_> {
    /// Encode the structured ORG value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: self
                .0
                .iter()
                .map(|unit| encode_component(&[unit.as_ref()]))
                .collect(),
        }
    }
}

impl VcardGeo<'_> {
    /// Encode the structured GEO value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![
                encode_component(&[self.latitude.as_ref()]),
                encode_component(&[self.longitude.as_ref()]),
            ],
        }
    }
}

impl VcardUnknownValue<'_> {
    /// Encode the raw components straight back into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: self
                .components
                .iter()
                .map(|c| encode_component(c))
                .collect(),
        }
    }
}

/// Serialize the decoded card by encoding it into a CST (canonical).
impl fmt::Display for Vcard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

/// A one-component, one-value syntax node, escaping the value.
fn scalar_node(value: &str) -> VcardValueNode<'static> {
    VcardValueNode {
        components: vec![encode_component(&[value])],
    }
}

/// Encode a clean value list into one owned component, escaping each value.
pub(crate) fn encode_component<S: AsRef<str>>(values: &[S]) -> Vec<VcardLeaf<'static>> {
    values
        .iter()
        .map(|v| VcardLeaf::from(escape(v.as_ref()).into_owned()))
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

/// Escape a value for a single value position: the separators `,` `;`, the
/// escape `\\` and newlines. Borrows when nothing needs escaping.
pub(crate) fn escape(text: &str) -> Cow<'_, str> {
    if !text
        .bytes()
        .any(|b| matches!(b, b'\\' | b',' | b';' | b'\n'))
    {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ',' => out.push_str("\\,"),
            ';' => out.push_str("\\;"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::v30::{
        tree::encode::escape,
        value::{n::VcardN, text::VcardText},
    };

    #[test]
    fn escapes_separators_and_newlines_and_borrows_when_clean() {
        assert_eq!(escape("a,b;c\nd"), r"a\,b\;c\nd");
        assert!(matches!(escape("plain"), Cow::Borrowed("plain")));
    }

    #[test]
    fn encodes_a_text_value_escaping_it() {
        let node = VcardText(Cow::Borrowed("hi, there")).encode();
        assert_eq!(node.to_string(), r"hi\, there");
    }

    #[test]
    fn encodes_the_structured_n_value_with_all_components() {
        let n = VcardN {
            family: vec![Cow::Borrowed("Doe")],
            ..Default::default()
        };
        assert_eq!(n.encode().to_string(), "Doe;;;;");
    }
}
