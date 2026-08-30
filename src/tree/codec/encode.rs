//! # Encode (model to syntax)
//!
//! The write side of the structural bridge: project the decoded model onto a
//! raw syntax tree.
//!
//! A value's [`VcardCodec`] impl encodes it into a [`VcardValueNode`], a
//! [`VcardParam`] into a [`VcardParamNode`], a [`VcardProp`] into a
//! [`VcardLine`] (its name verbatim from the property, its value delegated to
//! the value codec), and a [`Vcard`] into a whole [`VcardCst`].
//!
//! The card is encoded for its version's [`VcardEscaper`], which the value
//! codecs use to escape every leaf (through the sibling
//! [`escape`](crate::tree::codec::escape) codec) and to pick any
//! version-specific value shape.
//!
//! Byte-preserving edits are the cursors' job, not this module's.
//! [`Display`](core::fmt::Display) for [`Vcard`] renders a decoded card
//! straight to its serialized bytes through here.

use core::fmt;

use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

use crate::{
    param::VcardParam,
    prop::VcardProp,
    tree::{
        codec::{
            VcardCodec,
            escape::{escape_param, escape_with},
            mode::VcardEscaper,
        },
        cst::VcardCst,
        leaf::{VcardLeaf, VcardValueLeaf},
        line::VcardLine,
        param::node::VcardParamNode,
        value::node::VcardValueNode,
        wire::VcardWire,
    },
    vcard::Vcard,
};

impl Vcard<'_> {
    /// Encode the whole card into a CST for its version's escaping mode.
    pub fn encode(&self) -> VcardCst<'static> {
        let escaper = VcardEscaper::for_version(self.version);

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
    pub fn encode(&self, escaper: VcardEscaper) -> VcardLine<'static> {
        VcardLine {
            name: VcardLeaf::from(self.name.to_string()),
            params: self
                .params
                .iter()
                .map(|param| param.encode(escaper))
                .collect(),
            value: self.value.encode(escaper),
            eol: VcardLeaf::from("\r\n".to_string()),
            wire: VcardWire::default(),
        }
    }
}

impl VcardParam<'_> {
    /// Encode the parameter into a raw parameter node for the given escaping
    /// mode, dispatching on its kind.
    pub fn encode(&self, escaper: VcardEscaper) -> VcardParamNode<'static> {
        use crate::param::VcardParamKind::*;

        match self {
            VcardParam::Language(v) => param_scalar(&Language, v, escaper),
            VcardParam::Charset(v) => param_scalar(&Charset, v, escaper),
            VcardParam::Encoding(v) => param_scalar(&Encoding, v, escaper),
            VcardParam::Value(v) => param_scalar(&Value, v, escaper),
            VcardParam::Pref(v) => param_scalar(&Pref, v, escaper),
            VcardParam::AltId(v) => param_scalar(&AltId, v, escaper),
            VcardParam::Pid(vs) => param_list(&Pid, vs, escaper),
            VcardParam::Type(vs) => param_list(&Type, vs, escaper),
            VcardParam::MediaType(v) => param_scalar(&MediaType, v, escaper),
            VcardParam::CalScale(v) => param_scalar(&CalScale, v, escaper),
            VcardParam::SortAs(vs) => param_list(&SortAs, vs, escaper),
            VcardParam::Geo(v) => param_scalar(&Geo, v, escaper),
            VcardParam::Tz(v) => param_scalar(&Tz, v, escaper),
            VcardParam::Label(v) => param_scalar(&Label, v, escaper),
            VcardParam::Author(v) => param_scalar(&Author, v, escaper),
            VcardParam::AuthorName(v) => param_scalar(&AuthorName, v, escaper),
            VcardParam::Created(v) => param_scalar(&Created, v, escaper),
            VcardParam::Derived(v) => param_scalar(&Derived, v, escaper),
            VcardParam::Jsptr(v) => param_scalar(&Jsptr, v, escaper),
            VcardParam::Phonetic(v) => param_scalar(&Phonetic, v, escaper),
            VcardParam::PropId(v) => param_scalar(&PropId, v, escaper),
            VcardParam::Script(v) => param_scalar(&Script, v, escaper),
            VcardParam::ServiceType(v) => param_scalar(&ServiceType, v, escaper),
            VcardParam::Username(v) => param_scalar(&Username, v, escaper),

            VcardParam::Unknown { name, values } => param_list(name, values, escaper),
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
pub(crate) fn scalar_node(value: &str, escaper: VcardEscaper) -> VcardValueNode<'static> {
    VcardValueNode::from_components(vec![encode_component(&[value], escaper)], escaper)
}

/// Own one value exactly as given, with no escaping at all.
///
/// A URI is not text: RFC 6350 section 4.2 gives it no escapes, so escaping
/// its `;` or `,` on the way out would rewrite the reference the value is,
/// and a value that decoded whole would not survive its own round trip.
pub(crate) fn verbatim_node(value: &str, escaper: VcardEscaper) -> VcardValueNode<'static> {
    VcardValueNode::from_raw(value.as_bytes().to_vec(), escaper)
}

/// Escape and own a clean value list into one component, by escaping mode.
pub(crate) fn encode_component<S: AsRef<str>>(
    values: &[S],
    escaper: VcardEscaper,
) -> Vec<VcardValueLeaf<'static>> {
    values.iter().map(|v| encode_leaf(v, escaper)).collect()
}

/// Escape and own raw value bytes into one component, by escaping mode.
///
/// The foreign-charset escape hatch: only the structural separators are
/// escaped, every other byte going out exactly as given.
pub(crate) fn encode_bytes_component<B: AsRef<[u8]>>(
    values: &[B],
    escaper: VcardEscaper,
) -> Vec<VcardValueLeaf<'static>> {
    values
        .iter()
        .map(|v| VcardValueLeaf::from(escape_with(v.as_ref(), escaper).into_owned()))
        .collect()
}

/// Escape one value into an owned leaf, by escaping mode. Backs the per-item
/// value edits, which splice a single leaf and leave its siblings' bytes as
/// they were parsed.
pub(crate) fn encode_leaf<S: AsRef<str>>(
    value: S,
    escaper: VcardEscaper,
) -> VcardValueLeaf<'static> {
    VcardValueLeaf::from(escape_with(value.as_ref().as_bytes(), escaper).into_owned())
}

/// A parameter node from a single value, encoded by the given mode's parameter
/// rules.
fn param_scalar(name: &str, value: &str, escaper: VcardEscaper) -> VcardParamNode<'static> {
    VcardParamNode {
        name: VcardLeaf::from(name.to_string()),
        values: vec![VcardLeaf::from(escape_param(value, escaper).into_owned())],
        escaper,
    }
}

/// A parameter node from a value list, encoded by the given mode's parameter
/// rules.
fn param_list(
    name: &str,
    values: &[Cow<'_, str>],
    escaper: VcardEscaper,
) -> VcardParamNode<'static> {
    VcardParamNode {
        name: VcardLeaf::from(name.to_string()),
        values: values
            .iter()
            .map(|v| VcardLeaf::from(escape_param(v, escaper).into_owned()))
            .collect(),
        escaper,
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::{
        param::VcardParam,
        tree::{
            codec::{VcardCodec, mode::VcardEscaper},
            cst::VcardCst,
        },
        value::{n::VcardN, text::VcardText},
    };

    #[test]
    fn encodes_a_text_value_escaping_it() {
        let node = VcardText(Cow::Borrowed("hi, there")).encode(VcardEscaper::V4_0);
        assert_eq!(node.to_string(), r"hi\, there");
    }

    #[test]
    fn encodes_the_structured_n_value_with_all_components() {
        let n = VcardN {
            family: vec![Cow::Borrowed("Doe")],
            ..Default::default()
        };
        assert_eq!(n.encode(VcardEscaper::V4_0).to_string(), "Doe;;;;");
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

    #[test]
    fn encodes_the_rfc_6868_parameter_sequences() {
        // NOTE: RFC 6868 section 3.1 read backwards, over the three characters
        // a parameter value cannot carry raw.
        let param = VcardParam::Label(Cow::Borrowed("a\nb^c\"d"));

        assert_eq!(
            param.encode(VcardEscaper::V4_0).to_string(),
            "LABEL=a^nb^^c^'d",
        );
    }

    #[test]
    fn keeps_a_quoted_parameter_value_quoted() {
        // NOTE: the decoded model holds a parameter exactly as the wire spelled
        // it, its own delimiters included, so the surrounding pair is written
        // back as a pair rather than encoded as content.
        let param = VcardParam::Geo(Cow::Borrowed("\"geo:37.386,-122.083\""));

        assert_eq!(
            param.encode(VcardEscaper::V4_0).to_string(),
            "GEO=\"geo:37.386,-122.083\"",
        );
    }

    #[test]
    fn writes_a_pre_4_0_parameter_unencoded() {
        // NOTE: RFC 6868 updates RFC 6350 alone, so a 3.0 caret goes out as
        // itself.
        let param = VcardParam::Label(Cow::Borrowed("a^b"));

        assert_eq!(param.encode(VcardEscaper::V3_0).to_string(), "LABEL=a^b");
    }

    #[test]
    fn round_trips_a_parameter_byte_for_byte() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN;LANGUAGE=en;GEO=\"geo:37.386,-122.083\"",
            ";X-PATH=\"C:\\temp\";X-NOTE=a^nb^^c^'d:Ada\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();

        assert_eq!(cst.decode().to_string(), input);
    }
}
