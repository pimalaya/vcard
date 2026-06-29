//! # Encode (model to syntax)
//!
//! The write side of the bridge: project the decoded model onto a raw syntax
//! tree.
//!
//! `escape` is the write codec (it applies the 2.1 `;` value escape). On top
//! of it sit the `encode` methods, one per decoded type: each value type encodes
//! into a [`VcardValueNode`], a [`VcardParam`] encodes into a [`VcardParamNode`],
//! a [`VcardProp`] encodes into a [`VcardLine`] (its name taken verbatim from the
//! property, its value dispatched on the value kind), and a [`Vcard`] encodes
//! into a whole [`VcardCst`]. Binary values gain their declaring parameter
//! (`ENCODING=BASE64` or `VALUE=URL`) when missing. Encoding is always
//! canonical; byte-preserving edits are the cursors' job, not this module's.
//! [`Display`](core::fmt::Display) for [`Vcard`] renders a decoded card straight
//! to its serialized bytes through here.

use core::fmt;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::v21::{
    param::{VCARD_CHARSET, VCARD_ENCODING, VCARD_LANGUAGE, VCARD_VALUE, VcardParam},
    prop::VcardProp,
    tree::{
        cst::VcardCst, leaf::VcardLeaf, line::VcardLine, param::VcardParamNode,
        value::VcardValueNode,
    },
    value::{
        VcardUnknownValue, VcardValue,
        adr::VcardAdr,
        binary::VcardBinary,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        geo::VcardGeo,
        n::VcardN,
        org::VcardOrg,
        text::VcardText,
        uri::VcardUri,
    },
    vcard::Vcard,
    version::VCARD_VERSION,
};

impl Vcard<'_> {
    /// Encode the whole card into a canonical CST.
    pub fn encode(&self) -> VcardCst<'static> {
        let mut cst = VcardCst::v21();
        cst.version = VcardLine::text(VCARD_VERSION, self.version.as_str().to_string());
        cst.props = self.properties.iter().map(VcardProp::encode).collect();
        cst
    }
}

impl VcardProp<'_> {
    /// Encode the property into a raw content line, dispatching on its value.
    pub fn encode(&self) -> VcardLine<'static> {
        let mut params: Vec<VcardParamNode<'static>> =
            self.params.iter().flat_map(VcardParam::encode).collect();

        let value = match &self.value {
            VcardValue::Text(v) => v.encode(),
            VcardValue::Uri(v) => v.encode(),
            VcardValue::DateAndOrTime(v) => v.encode(),
            VcardValue::Timestamp(v) => v.encode(),
            VcardValue::N(v) => v.encode(),
            VcardValue::Adr(v) => v.encode(),
            VcardValue::Org(v) => v.encode(),
            VcardValue::Geo(v) => v.encode(),
            VcardValue::Binary(v) => {
                ensure_binary_param(&mut params, v);
                v.encode()
            }
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
    /// Encode the parameter into raw parameter nodes, dispatching on its kind.
    /// Most kinds map to a single `name=value` node; `Type` expands to one bare
    /// token per value (the 2.1 idiom `;WORK;VOICE`, not the 3.0 comma list
    /// `;TYPE=WORK,VOICE`).
    pub fn encode(&self) -> Vec<VcardParamNode<'static>> {
        match self {
            VcardParam::Charset(v) => vec![param_scalar(VCARD_CHARSET, v)],
            VcardParam::Encoding(v) => vec![param_scalar(VCARD_ENCODING, v)],
            VcardParam::Language(v) => vec![param_scalar(VCARD_LANGUAGE, v)],
            VcardParam::Value(v) => vec![param_scalar(VCARD_VALUE, v)],
            VcardParam::Type(vs) => vs.iter().map(|v| bare_token(v)).collect(),

            VcardParam::Unknown { name, values } => vec![VcardParamNode {
                name: VcardLeaf::from(name.to_string()),
                values: values.iter().map(|v| escaped_leaf(v)).collect(),
            }],
        }
    }
}

impl VcardValueNode<'_> {
    /// Set the `i`th component, escaping the value. Pads with empty components
    /// when needed; every other component is left untouched, so a parsed card
    /// keeps its bytes.
    pub fn set_at(&mut self, i: usize, value: &str) {
        while self.components.len() <= i {
            self.components.push(VcardLeaf::from(String::new()));
        }

        self.components[i] = escaped_leaf(value);
    }
}

impl VcardText<'_> {
    /// Encode a single text value into a syntax node. A 2.1 simple text value is
    /// not compound, so its `;` is written literally rather than escaped.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![raw_leaf(&self.0)],
        }
    }
}

impl VcardUri<'_> {
    /// Encode a URI value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![raw_leaf(&self.0)],
        }
    }
}

impl VcardDateAndOrTime<'_> {
    /// Encode a date-and-or-time value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![raw_leaf(&self.0)],
        }
    }
}

impl VcardTimestamp<'_> {
    /// Encode a timestamp value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![raw_leaf(&self.0)],
        }
    }
}

impl VcardN<'_> {
    /// Encode the structured N value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![
                escaped_leaf(&self.family),
                escaped_leaf(&self.given),
                escaped_leaf(&self.additional),
                escaped_leaf(&self.prefix),
                escaped_leaf(&self.suffix),
            ],
        }
    }
}

impl VcardAdr<'_> {
    /// Encode the structured ADR value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: vec![
                escaped_leaf(&self.po_box),
                escaped_leaf(&self.extended),
                escaped_leaf(&self.street),
                escaped_leaf(&self.locality),
                escaped_leaf(&self.region),
                escaped_leaf(&self.postal_code),
                escaped_leaf(&self.country),
            ],
        }
    }
}

impl VcardGeo<'_> {
    /// Encode the GEO value into a syntax node: `latitude,longitude` (vCard 2.1
    /// joins the pair with a comma, not a semicolon).
    pub fn encode(&self) -> VcardValueNode<'static> {
        let mut pair = String::with_capacity(self.latitude.len() + 1 + self.longitude.len());
        pair.push_str(&self.latitude);
        pair.push(',');
        pair.push_str(&self.longitude);

        VcardValueNode {
            components: vec![VcardLeaf::from(pair)],
        }
    }
}

impl VcardOrg<'_> {
    /// Encode the structured ORG value into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: self.0.iter().map(|unit| escaped_leaf(unit)).collect(),
        }
    }
}

impl VcardBinary<'_> {
    /// Encode a binary value into a syntax node, keeping the raw text verbatim.
    pub fn encode(&self) -> VcardValueNode<'static> {
        let raw = match self {
            VcardBinary::Uri(uri) => uri,
            VcardBinary::Base64(base64) => base64,
        };

        VcardValueNode {
            components: vec![raw_leaf(raw)],
        }
    }
}

impl VcardUnknownValue<'_> {
    /// Encode the raw components straight back into a syntax node.
    pub fn encode(&self) -> VcardValueNode<'static> {
        VcardValueNode {
            components: self.components.iter().map(|c| escaped_leaf(c)).collect(),
        }
    }
}

/// Serialize the decoded card by encoding it into a CST (canonical).
impl fmt::Display for Vcard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

/// Add the parameter a binary value needs to declare its form, unless the line
/// already carries it: `ENCODING=BASE64` for inline data, `VALUE=URL` for a
/// reference.
fn ensure_binary_param(params: &mut Vec<VcardParamNode<'static>>, binary: &VcardBinary<'_>) {
    match binary {
        VcardBinary::Base64(_) => {
            if !params
                .iter()
                .any(|p| p.name.get().eq_ignore_ascii_case(VCARD_ENCODING))
            {
                params.push(param_scalar(VCARD_ENCODING, "BASE64"));
            }
        }
        VcardBinary::Uri(_) => {
            if !params
                .iter()
                .any(|p| p.name.get().eq_ignore_ascii_case(VCARD_VALUE))
            {
                params.push(param_scalar(VCARD_VALUE, "URL"));
            }
        }
    }
}

/// A leaf holding an escaped value (the 2.1 `;` escape).
fn escaped_leaf(value: &str) -> VcardLeaf<'static> {
    VcardLeaf::from(escape(value).into_owned())
}

/// A leaf holding a value verbatim (no escaping).
fn raw_leaf(value: &str) -> VcardLeaf<'static> {
    VcardLeaf::from(value.to_string())
}

/// A parameter node from a single value, escaping a `;` in the value (2.1
/// requires a `\;` in a parameter value).
fn param_scalar(name: &str, value: &str) -> VcardParamNode<'static> {
    VcardParamNode {
        name: VcardLeaf::from(name.to_string()),
        values: vec![escaped_leaf(value)],
    }
}

/// A bare 2.1 parameter token: a name with no `=value`, escaping a `;` in it.
fn bare_token(value: &str) -> VcardParamNode<'static> {
    VcardParamNode {
        name: escaped_leaf(value),
        values: Vec::new(),
    }
}

/// Escape the 2.1 value separator `;` only (`;` -> `\;`). Borrows when clean.
pub(crate) fn escape(text: &str) -> Cow<'_, str> {
    if !text.contains(';') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len() + 2);

    for c in text.chars() {
        if c == ';' {
            out.push('\\');
        }
        out.push(c);
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::v21::{
        param::VcardParam,
        tree::{encode::escape, line::VcardLine},
        value::{geo::VcardGeo, n::VcardN, text::VcardText},
    };

    #[test]
    fn escapes_the_semicolon_and_borrows_when_clean() {
        assert_eq!(escape("a;b"), r"a\;b");
        assert!(matches!(escape("plain"), Cow::Borrowed("plain")));
    }

    #[test]
    fn writes_a_simple_text_value_literally() {
        let node = VcardText(Cow::Borrowed("a;b")).encode();
        assert_eq!(node.to_string(), "a;b");
    }

    #[test]
    fn encodes_the_structured_n_value_with_all_components() {
        let n = VcardN {
            family: Cow::Borrowed("Doe"),
            ..Default::default()
        };
        assert_eq!(n.encode().to_string(), "Doe;;;;");
    }

    #[test]
    fn encodes_geo_as_a_comma_pair() {
        let geo = VcardGeo {
            latitude: Cow::Borrowed("37.0"),
            longitude: Cow::Borrowed("-122.0"),
        };
        assert_eq!(geo.encode().to_string(), "37.0,-122.0");
    }

    #[test]
    fn encodes_a_type_value_as_a_bare_token_escaping_it() {
        let nodes = VcardParam::Type(vec![Cow::Borrowed("a;b")]).encode();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].to_string(), r"a\;b");
    }

    #[test]
    fn a_multi_value_type_round_trips_through_bare_tokens() {
        // a 3.0-style comma list is read leniently...
        let (line, _) = VcardLine::take("TEL;TYPE=WORK,VOICE:123\r\n").unwrap();
        let prop = line.decode();
        // ...and re-encoded in the 2.1 bare idiom.
        assert_eq!(prop.encode().to_string(), "TEL;WORK;VOICE:123\r\n");
    }
}
