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
//! [`Display`](core::fmt::Display) for [`Vcard`] writes a decoded card
//! straight to the text its encoded tree serializes to, with no tree built:
//! each value type spells its wire shape once, for a node and for text alike,
//! through the one writer both share.

use core::{fmt, mem, slice};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    param::VcardParam,
    prop::VcardProp,
    tree::{
        codec::{
            VcardCodec,
            escape::{escape_into, escape_param, escape_param_into, escape_with},
            mode::VcardEscaper,
        },
        cst::VcardCst,
        leaf::{VcardLeaf, VcardValueLeaf},
        line::VcardLine,
        param::node::VcardParamNode,
        value::node::VcardValueNode,
        wire::VcardWire,
    },
    validator::VcardValid,
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

impl From<VcardValid<Vcard<'_>>> for VcardCst<'static> {
    fn from(card: VcardValid<Vcard<'_>>) -> Self {
        card.into_inner().encode()
    }
}

impl VcardProp<'_> {
    /// Encode the property into a raw content line for the given escaping mode,
    /// dispatching on its value.
    pub fn encode(&self, escaper: VcardEscaper) -> VcardLine<'static> {
        let mut name = String::new();

        if let Some(group) = &self.group {
            name.push_str(group);
            name.push('.');
        }

        name.push_str(&self.name);

        VcardLine {
            name: VcardLeaf::from(name),
            params: self
                .params
                .iter()
                .map(|param| param.encode(escaper))
                .collect(),
            value: self.value.encode(escaper),
            eol: VcardLeaf::from("\r\n"),
            wire: VcardWire::default(),
        }
    }

    /// Append the line [`encode`](Self::encode) builds, as its `Display`
    /// writes it.
    fn write_text(&self, escaper: VcardEscaper, text: &mut String) {
        if let Some(group) = &self.group {
            text.push_str(group);
            text.push('.');
        }

        text.push_str(&self.name);

        for param in &self.params {
            text.push(';');
            param.write_text(escaper, text);
        }

        text.push(':');

        self.value.emit(&mut VcardValueOut {
            escaper,
            sink: Sink::Text {
                text,
                started: false,
            },
        });

        text.push_str("\r\n");
    }
}

impl VcardParam<'_> {
    /// Encode the parameter into a raw parameter node for the given escaping
    /// mode, dispatching on its kind.
    pub fn encode(&self, escaper: VcardEscaper) -> VcardParamNode<'static> {
        let (name, values) = self.wire();

        VcardParamNode {
            name: VcardLeaf::from(name.to_string()),
            values: values
                .iter()
                .map(|value| VcardLeaf::from(escape_param(value, escaper).into_owned()))
                .collect(),
            escaper,
        }
    }

    /// Append the node [`encode`](Self::encode) builds, as its `Display`
    /// writes it.
    fn write_text(&self, escaper: VcardEscaper, text: &mut String) {
        let (name, values) = self.wire();

        text.push_str(name);

        for (i, value) in values.iter().enumerate() {
            text.push(if i == 0 { '=' } else { ',' });
            escape_param_into(value, escaper, text);
        }
    }

    /// The parameter's wire name and its values, not yet encoded: a scalar
    /// kind holds one.
    fn wire(&self) -> (&str, &[Cow<'_, str>]) {
        use crate::param::VcardParamKind::*;

        match self {
            VcardParam::Language(v) => (&Language, slice::from_ref(v)),
            VcardParam::Charset(v) => (&Charset, slice::from_ref(v)),
            VcardParam::Encoding(v) => (&Encoding, slice::from_ref(v)),
            VcardParam::Value(v) => (&Value, slice::from_ref(v)),
            VcardParam::Pref(v) => (&Pref, slice::from_ref(v)),
            VcardParam::AltId(v) => (&AltId, slice::from_ref(v)),
            VcardParam::Pid(vs) => (&Pid, vs),
            VcardParam::Type(vs) => (&Type, vs),
            VcardParam::MediaType(v) => (&MediaType, slice::from_ref(v)),
            VcardParam::CalScale(v) => (&CalScale, slice::from_ref(v)),
            VcardParam::SortAs(vs) => (&SortAs, vs),
            VcardParam::Geo(v) => (&Geo, slice::from_ref(v)),
            VcardParam::Tz(v) => (&Tz, slice::from_ref(v)),
            VcardParam::Label(v) => (&Label, slice::from_ref(v)),
            VcardParam::Author(v) => (&Author, slice::from_ref(v)),
            VcardParam::AuthorName(v) => (&AuthorName, slice::from_ref(v)),
            VcardParam::Created(v) => (&Created, slice::from_ref(v)),
            VcardParam::Derived(v) => (&Derived, slice::from_ref(v)),
            VcardParam::Jsptr(v) => (&Jsptr, slice::from_ref(v)),
            VcardParam::Phonetic(v) => (&Phonetic, slice::from_ref(v)),
            VcardParam::PropId(v) => (&PropId, slice::from_ref(v)),
            VcardParam::Script(v) => (&Script, slice::from_ref(v)),
            VcardParam::ServiceType(v) => (&ServiceType, slice::from_ref(v)),
            VcardParam::Username(v) => (&Username, slice::from_ref(v)),

            VcardParam::Unknown { name, values } => (name, values),
        }
    }
}

/// Serialize the decoded card as its [`encode`](Vcard::encode)d tree
/// serializes, written straight to text.
impl fmt::Display for Vcard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let escaper = VcardEscaper::for_version(self.version);

        // NOTE: Sized for the envelope and a short line per property, which
        // most are; a long value grows the text once, by its own length.
        let mut text = String::with_capacity(64 + 32 * self.properties.len());

        text.push_str("BEGIN:VCARD\r\nVERSION:");
        text.push_str(&self.version);
        text.push_str("\r\n");

        for prop in &self.properties {
            prop.write_text(escaper, &mut text);
        }

        text.push_str("END:VCARD\r\n");
        f.write_str(&text)
    }
}

/// Where a value's wire shape goes: the leaves of a syntax node, as
/// [`VcardCodec::encode`] builds it, or the text that node serializes to.
///
/// A value type spells its shape once, in its `emit`, as one
/// [`verbatim`](Self::verbatim) value or a run of
/// [`component`](Self::component)s, so the two ways out cannot drift apart.
pub(crate) struct VcardValueOut<'o> {
    /// The escaping mode, which also picks a version-specific shape.
    pub(crate) escaper: VcardEscaper,
    /// What the shape is written into.
    sink: Sink<'o>,
}

impl VcardValueOut<'_> {
    /// Build a value's node from what `emit` writes.
    pub(crate) fn node(
        escaper: VcardEscaper,
        emit: impl FnOnce(&mut VcardValueOut<'_>),
    ) -> VcardValueNode<'static> {
        let mut node = VcardValueNode::from_components(Vec::new(), escaper);

        emit(&mut VcardValueOut {
            escaper,
            sink: Sink::Node(&mut node),
        });

        node
    }

    /// Write one value exactly as given, with no escaping at all.
    ///
    /// A URI is not text: RFC 6350 section 4.2 gives it no escapes, so
    /// escaping its `;` or `,` on the way out would rewrite the reference the
    /// value is, and a value that decoded whole would not survive its own
    /// round trip.
    pub(crate) fn verbatim(&mut self, value: &str) {
        match &mut self.sink {
            Sink::Node(node) => {
                **node = VcardValueNode::from_raw(value.as_bytes().to_vec(), self.escaper);
            }
            Sink::Text { text, .. } => text.push_str(value),
        }
    }

    /// Write one `;`-separated component of `,`-separated values, each
    /// escaped by the mode.
    pub(crate) fn component<S: AsRef<str>>(&mut self, values: &[S]) {
        match &mut self.sink {
            Sink::Node(node) => node.set_component(node.component_count(), values),
            Sink::Text { text, started } => {
                if mem::replace(started, true) {
                    text.push(';');
                }

                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        text.push(',');
                    }

                    escape_into(value.as_ref(), self.escaper, text);
                }
            }
        }
    }
}

/// What a [`VcardValueOut`] writes into.
enum Sink<'o> {
    /// A node, its components pushed one by one, or replaced whole by a
    /// verbatim value.
    Node(&'o mut VcardValueNode<'static>),
    /// Text, `started` once a component is written, so the next one goes
    /// after a `;`.
    Text { text: &'o mut String, started: bool },
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

    /// A 2.1 GEO decodes to a coordinate pair, and re-encoding for 2.1 must
    /// write it back as a comma pair rather than the 3.0 semicolon form. The
    /// same pair round-trips through 3.0 with a semicolon.
    #[test]
    fn encodes_the_geo_pair_in_the_cards_own_version() {
        let cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nGEO:37.0,-122.0\r\nEND:VCARD\r\n")
            .unwrap();
        let card = cst.decode();
        assert!(
            card.to_string().contains("GEO:37.0,-122.0\r\n"),
            "{}",
            card.to_string(),
        );

        let cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:3.0\r\nGEO:37.0;-122.0\r\nEND:VCARD\r\n")
            .unwrap();
        let card = cst.decode();
        assert!(
            card.to_string().contains("GEO:37.0;-122.0\r\n"),
            "{}",
            card.to_string(),
        );
    }

    /// RFC 6868 section 3.1 read backwards, over the three characters a
    /// parameter value cannot carry raw.
    #[test]
    fn encodes_the_rfc_6868_parameter_sequences() {
        let param = VcardParam::Label(Cow::Borrowed("a\nb^c\"d"));

        assert_eq!(
            param.encode(VcardEscaper::V4_0).to_string(),
            "LABEL=a^nb^^c^'d",
        );
    }

    /// The decoded model holds a parameter's content, its RFC 6350 section 3.3
    /// delimiters excluded, so the pair is put back around a value carrying a
    /// character a bare SAFE-CHAR run may not hold.
    #[test]
    fn quotes_a_parameter_value_carrying_a_delimiter() {
        let param = VcardParam::Geo(Cow::Borrowed("geo:37.386,-122.083"));

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
