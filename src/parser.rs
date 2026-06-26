//! Parsing vCard text into an edition-ready tree of byte-range leaves.
//!
//! [`VcardTree`] owns or borrows the source as a [`Cow`] and keeps each
//! property as byte-range [`VcardLeaf`](leaf::VcardLeaf)s plus an optional
//! per-leaf override. Rebuilding copies the source verbatim and splices in only
//! the overridden ranges, so untouched bytes survive exactly and edits stay
//! precise.
//!
//! A property is reached by type through the [`VcardPropLens`] trait
//! (`card.prop_mut::<N>()`), and its parameters the same way through the
//! [`VcardParamLens`](param::lens::VcardParamLens) trait
//! (`...param_mut::<PID>()`); parameters are a `Vec` that can hold any of them,
//! exactly like properties. The whole tree is walked by [`VcardTree::nodes`],
//! an explicit-stack iterator that never recurses. The properties and
//! parameters that need a custom representation get their own module under
//! [`prop`] and [`param`]; everything else stays generic.

pub mod decode;
pub mod leaf;
pub mod line;
pub mod node;
pub mod param;
pub mod prop;
pub mod utils;
pub mod value;

use core::fmt;

use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::{
    error::VcardParseError,
    parser::{
        leaf::VcardLeaf,
        line::VcardLine,
        node::{VcardNode, VcardNodes},
        prop::{
            lens::VcardPropLens, node::VcardPropNode, view::VcardPropView,
            view_mut::VcardPropViewMut,
        },
        utils::property_name,
    },
    rfc6350::{
        vcard::{BEGIN, END},
        version::{VERSION, VcardVersion},
    },
};

/// A parsed card: the source it owns or borrows, its version and properties.
pub struct VcardTree<'a> {
    /// The source text the leaf ranges point into.
    pub input: Cow<'a, str>,
    /// The card version.
    pub version: VcardVersion,
    /// The properties, in source order.
    pub props: Vec<VcardPropNode>,
}

impl<'a> VcardTree<'a> {
    /// Parse exactly one card. Pass a `&str` to borrow the source or a `String`
    /// to own it; the resulting tree is valid either way.
    pub fn parse(input: impl Into<Cow<'a, str>>) -> Result<Self, VcardParseError> {
        let input = input.into();

        let mut version = VcardVersion::default();
        let mut properties = Vec::new();
        let mut state = State::Begin;
        let mut offset = 0;

        while offset < input.len() {
            let line = VcardLine::parse(&input, offset)?;
            let name = property_name(&input[line.prop.clone()]);
            let value = &input[line.value.clone()];

            match state {
                State::Begin => {
                    if !name.eq_ignore_ascii_case(BEGIN) {
                        return Err(VcardParseError::ExpectedBegin(name.to_string()));
                    }

                    state = State::Version;
                }
                State::Version => {
                    if !name.eq_ignore_ascii_case(VERSION) {
                        return Err(VcardParseError::ExpectedVersion(name.to_string()));
                    }

                    version = match value {
                        "2.1" => VcardVersion::V2_1,
                        "3.0" => VcardVersion::V3_0,
                        "4.0" => VcardVersion::V4_0,
                        v => return Err(VcardParseError::UnsupportedVersion(v.to_string())),
                    };

                    state = State::Property;
                }
                State::Property => {
                    if name.eq_ignore_ascii_case(END) {
                        return Ok(Self {
                            input,
                            version,
                            props: properties,
                        });
                    }

                    properties.push(VcardPropNode::parse(&input, &line));
                }
            }

            offset = line.crlf.end;
        }

        Err(VcardParseError::MissingEnd(input.to_string()))
    }

    /// A depth-first walk over every leaf in the card, using an explicit stack
    /// rather than recursion.
    pub fn nodes(&self) -> VcardNodes<'_> {
        VcardNodes {
            stack: self.props.iter().rev().map(VcardNode::Property).collect(),
        }
    }

    /// The first property of type `L` (for example `card.prop::<N>()`), as a
    /// typed view over its name, parameters and value.
    pub fn prop<L: VcardPropLens>(&self) -> Option<VcardPropView<'_, L::Target>> {
        self.props.iter().find_map(|property| {
            let input = &self.input;

            if !property.name.text(input).eq_ignore_ascii_case(L::NAME) {
                return None;
            }

            Some(VcardPropView {
                input,
                name: &property.name,
                params: &property.params,
                value: L::get(&property.value)?,
            })
        })
    }

    /// The first property of type `L`, as a fully mutable typed view.
    pub fn prop_mut<L: VcardPropLens>(&mut self) -> Option<VcardPropViewMut<'_, L::Target>> {
        self.props.iter_mut().find_map(|prop| {
            let input = &self.input;

            if !prop.name.text(input).eq_ignore_ascii_case(L::NAME) {
                return None;
            }

            Some(VcardPropViewMut {
                input,
                name: &mut prop.name,
                params: &mut prop.params,
                value: L::get_mut(&mut prop.value)?,
            })
        })
    }
}

impl fmt::Display for VcardTree<'_> {
    /// Rebuild the card text: the source verbatim, with each overridden leaf
    /// spliced in over its original bytes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut edits: Vec<_> = self.nodes().filter_map(VcardLeaf::edit).collect();
        edits.sort_by_key(|(range, _)| range.start);

        let input: &str = &self.input;
        let mut cursor = 0;

        for (range, text) in edits {
            write!(f, "{}", &input[cursor..range.start])?;
            write!(f, "{text}")?;
            cursor = range.end;
        }

        write!(f, "{}", &input[cursor..])
    }
}

/// The state machine position while walking a card.
#[derive(Clone, Copy)]
enum State {
    /// Expecting a BEGIN line to open the card.
    Begin,
    /// Expecting the VERSION line right after BEGIN.
    Version,
    /// Collecting property lines until an END line.
    Property,
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString};

    use crate::{
        parser::{
            VcardTree,
            param::pid::PID,
            prop::{adr::ADR, kind::KIND, n::N},
            value::VcardValueNode,
        },
        rfc6350::prop::kind::VcardKind,
    };

    const CARD: &str = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "N:Doe;John;;Dr.;\r\n",
        "ADR:;;1 Main St;Town;;12345;US\r\n",
        "KIND:individual\r\n",
        "FN:John Doe\r\n",
        "END:VCARD\r\n",
    );

    #[test]
    fn round_trips_byte_for_byte() {
        let card = VcardTree::parse(CARD).unwrap();
        assert_eq!(card.to_string(), CARD);
    }

    #[test]
    fn decomposes_adr_into_seven_components() {
        let card = VcardTree::parse(CARD).unwrap();

        let VcardValueNode::Address(adr) = &card.props[1].value else {
            panic!("expected ADR as the second property");
        };

        assert_eq!(adr.street[0].text(CARD), "1 Main St");
        assert_eq!(adr.locality[0].text(CARD), "Town");
        assert_eq!(adr.postal_code[0].text(CARD), "12345");
        assert_eq!(adr.country[0].text(CARD), "US");
        assert_eq!(adr.po_box[0].text(CARD), "");
    }

    #[test]
    fn edits_an_adr_component() {
        let mut card = VcardTree::parse(CARD).unwrap();

        let adr = card.prop_mut::<ADR>().unwrap();
        adr.value.locality[0].replace("Newtown");

        assert!(
            card.to_string()
                .contains("ADR:;;1 Main St;Newtown;;12345;US\r\n")
        );
    }

    #[test]
    fn decodes_kind_into_the_enum() {
        let card = VcardTree::parse(CARD).unwrap();

        let kind = card.prop::<KIND>().unwrap().decode();
        assert_eq!(kind, VcardKind::Individual);

        let other = VcardTree::parse(concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "KIND:x-robot\r\n",
            "END:VCARD\r\n",
        ))
        .unwrap();
        let kind = other.prop::<KIND>().unwrap().decode();
        assert_eq!(kind, VcardKind::Other(Cow::Borrowed("x-robot")));
    }

    #[test]
    fn typed_get_and_decode_for_n() {
        let card = VcardTree::parse(CARD).unwrap();

        let n = card.prop::<N>().expect("an N property");
        assert_eq!(n.value.given[0].text(CARD), "John");

        let name = n.decode();
        assert_eq!(name.family[0].as_ref(), "Doe");
    }

    #[test]
    fn selects_a_parameter_by_type() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "N;PID=1:Doe;John;;;\r\n",
            "END:VCARD\r\n",
        );

        let mut card = VcardTree::parse(input).unwrap();
        let mut n = card.prop_mut::<N>().unwrap();
        n.param_mut::<PID>().expect("a PID parameter").values[0].replace("2");

        assert!(card.to_string().contains("N;PID=2:Doe;John;;;\r\n"));
    }

    #[test]
    fn the_leaf_walk_visits_every_leaf() {
        let card = VcardTree::parse(CARD).unwrap();
        // N(name+5) + ADR(name+7) + KIND(name+1) + FN(name+1) = 6+8+2+2
        assert_eq!(card.nodes().count(), 18);
    }
}
