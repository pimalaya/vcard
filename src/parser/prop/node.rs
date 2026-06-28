use core::fmt::{self, Display, Formatter};

use alloc::vec::Vec;

use crate::{
    parser::{
        leaf::VcardLeaf,
        param::node::VcardParamNode,
        prop::{adr::VcardAddressNode, kind::VcardKindNode, n::VcardNameNode},
        utils::parse_params,
        value::VcardValueNode,
    },
    rfc6350::prop::{adr::ADR, kind::KIND, n::N},
};

/// One parsed content line: its name leaf, its parameters, its value and the
/// line ending. Serializing emits the invariant separators (`;` `=` `,` `:`)
/// between them, so a parsed line round-trips byte for byte.
pub struct VcardPropNode<'a> {
    /// The property name leaf, with any group prefix.
    pub name: VcardLeaf<'a>,
    /// The parameters, in source order; any parameter may appear.
    pub params: Vec<VcardParamNode<'a>>,
    /// The value, decomposed when the property is modelled.
    pub value: VcardValueNode<'a>,
    /// The line ending that terminated the line (`\r\n` or `\n`).
    pub eol: VcardLeaf<'a>,
}

impl<'a> VcardPropNode<'a> {
    pub(crate) fn parse(head: &'a str, value: &'a str, eol: &'a str) -> Self {
        let (name, params) = parse_params(head);

        let value = if name.eq_ignore_ascii_case(N) {
            VcardValueNode::Name(VcardNameNode::parse(value))
        } else if name.eq_ignore_ascii_case(ADR) {
            VcardValueNode::Address(VcardAddressNode::parse(value))
        } else if name.eq_ignore_ascii_case(KIND) {
            VcardValueNode::Kind(VcardKindNode::parse(value))
        } else {
            VcardValueNode::Leaf(VcardLeaf::from(value))
        };

        Self {
            name: VcardLeaf::from(name),
            params,
            value,
            eol: VcardLeaf::from(eol),
        }
    }
}

impl Display for VcardPropNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.text())?;

        for param in &self.params {
            write!(f, ";{param}")?;
        }

        write!(f, ":{}{}", self.value, self.eol.text())
    }
}
