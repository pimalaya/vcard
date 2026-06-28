//! Parsing vCard text into an edition-ready, full-fidelity tree of leaves.
//!
//! [`VcardTree`] borrows the source `&str` and keeps each content line as
//! [`VcardLeaf`](leaf::VcardLeaf)s holding a [`Cow`](alloc::borrow::Cow):
//! borrowed slices when parsed, owned strings when built or edited. Serializing
//! emits the invariant separators (`;` `=` `,` `:`) a node sits between, and
//! keeps each line ending verbatim, so a parsed card rebuilds byte for byte and
//! edits stay precise. The BEGIN, VERSION and END lines are kept as nodes too,
//! so nothing is reconstructed by rule.
//!
//! A property is reached by type through the
//! [`VcardPropLens`] trait (`card.prop_mut::<N>()`),
//! and its parameters the same way through the
//! [`VcardParamLens`](param::lens::VcardParamLens) trait (`...param_mut::<PID>()`);
//! parameters are a `Vec` that can hold any of them, exactly like properties.
//! The properties and parameters that need a custom representation get their own
//! module under [`prop`] and [`param`]; everything else stays generic.

pub mod decode;
pub mod leaf;
pub mod line;
pub mod param;
pub mod prop;
pub mod utils;
pub mod value;

use core::fmt::{self, Display, Formatter};

use alloc::{string::ToString, vec::Vec};

use crate::{
    error::VcardParseError,
    parser::{
        line::VcardLine,
        prop::{
            lens::VcardPropLens, node::VcardPropNode, view::VcardPropView,
            view_mut::VcardPropViewMut,
        },
        utils::property_name,
    },
    rfc6350::{
        vcard::{BEGIN, END},
        version::VERSION,
    },
};

/// A parsed card: its BEGIN, VERSION and END lines, and the property lines in
/// between, all borrowing the source the leaves point at.
pub struct VcardTree<'a> {
    /// The BEGIN line opening the card.
    pub begin: VcardPropNode<'a>,
    /// The VERSION line following BEGIN.
    pub version: VcardPropNode<'a>,
    /// The property lines, in source order.
    pub props: Vec<VcardPropNode<'a>>,
    /// The END line closing the card.
    pub end: VcardPropNode<'a>,
}

impl<'a> VcardTree<'a> {
    /// Parse exactly one card, borrowing `input` for the tree's lifetime.
    pub fn parse(input: &'a str) -> Result<Self, VcardParseError> {
        let (line, mut rest) = VcardLine::parse(input)?;

        if !property_name(line.head).eq_ignore_ascii_case(BEGIN) {
            return Err(VcardParseError::ExpectedBegin(line.head.to_string()));
        }

        let begin = VcardPropNode::parse(line.head, line.value, line.eol);

        let (line, tail) = VcardLine::parse(rest)?;
        rest = tail;

        if !property_name(line.head).eq_ignore_ascii_case(VERSION) {
            return Err(VcardParseError::ExpectedVersion(line.head.to_string()));
        }

        match line.value {
            "2.1" | "3.0" | "4.0" => {}
            other => return Err(VcardParseError::UnsupportedVersion(other.to_string())),
        }

        let version = VcardPropNode::parse(line.head, line.value, line.eol);

        let mut props = Vec::new();

        loop {
            if rest.is_empty() {
                return Err(VcardParseError::MissingEnd(input.to_string()));
            }

            let (line, tail) = VcardLine::parse(rest)?;
            rest = tail;

            let node = VcardPropNode::parse(line.head, line.value, line.eol);

            if property_name(line.head).eq_ignore_ascii_case(END) {
                return Ok(Self {
                    begin,
                    version,
                    props,
                    end: node,
                });
            }

            props.push(node);
        }
    }

    /// The first property of type `L` (for example `card.prop::<N>()`), as a
    /// typed view over its name, parameters and value.
    pub fn prop<L: VcardPropLens>(&self) -> Option<VcardPropView<'_, 'a, L::Target<'a>>> {
        self.props.iter().find_map(|prop| {
            if !prop.name.text().eq_ignore_ascii_case(L::NAME) {
                return None;
            }

            L::get(&prop.value).map(|value| VcardPropView {
                name: &prop.name,
                params: &prop.params,
                value,
            })
        })
    }

    /// The first property of type `L`, as a fully mutable typed view.
    pub fn prop_mut<L: VcardPropLens>(
        &mut self,
    ) -> Option<VcardPropViewMut<'_, 'a, L::Target<'a>>> {
        self.props.iter_mut().find_map(|prop| {
            if !prop.name.text().eq_ignore_ascii_case(L::NAME) {
                return None;
            }

            L::get_mut(&mut prop.value).map(|value| VcardPropViewMut {
                name: &mut prop.name,
                params: &mut prop.params,
                value,
            })
        })
    }
}

impl Display for VcardTree<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.begin, self.version)?;

        for prop in &self.props {
            write!(f, "{prop}")?;
        }

        write!(f, "{}", self.end)
    }
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

        assert_eq!(adr.street[0].text(), "1 Main St");
        assert_eq!(adr.locality[0].text(), "Town");
        assert_eq!(adr.postal_code[0].text(), "12345");
        assert_eq!(adr.country[0].text(), "US");
        assert_eq!(adr.po_box[0].text(), "");
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
        assert_eq!(n.value.given[0].text(), "John");

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
}
