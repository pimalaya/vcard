use core::fmt::{self, Display, Formatter};

use crate::parser::{
    leaf::VcardLeaf,
    prop::{adr::VcardAddressNode, kind::VcardKindNode, n::VcardNameNode},
};

/// A property value: typed components when the property is modelled, otherwise
/// the whole value as one raw leaf.
pub enum VcardValueNode<'a> {
    /// The N property, as its five name components.
    Name(VcardNameNode<'a>),
    /// The ADR property, as its seven address components.
    Address(VcardAddressNode<'a>),
    /// The KIND property, as a single value leaf.
    Kind(VcardKindNode<'a>),
    /// Any other property: the value as one raw leaf.
    Leaf(VcardLeaf<'a>),
}

impl Display for VcardValueNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            VcardValueNode::Name(name) => write!(f, "{name}"),
            VcardValueNode::Address(adr) => write!(f, "{adr}"),
            VcardValueNode::Kind(kind) => write!(f, "{kind}"),
            VcardValueNode::Leaf(leaf) => f.write_str(leaf.text()),
        }
    }
}
