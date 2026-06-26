use alloc::{vec, vec::Vec};

use crate::parser::{
    leaf::VcardLeaf,
    prop::{adr::VcardAddressNode, kind::VcardKindNode, n::VcardNameNode},
};

/// A property value: typed leaves when the property is modelled, otherwise
/// the whole value as one raw leaf.
pub enum VcardValueNode {
    /// The N property, as its five name components.
    Name(VcardNameNode),
    /// The ADR property, as its seven address components.
    Address(VcardAddressNode),
    /// The KIND property, as a single value leaf.
    Kind(VcardKindNode),
    /// Any other property: the value as one leaf.
    Leaf(VcardLeaf),
}

impl VcardValueNode {
    /// Every leaf of the value, in source order.
    pub(crate) fn leaves(&self) -> Vec<&VcardLeaf> {
        match self {
            VcardValueNode::Name(name) => name.leaves(),
            VcardValueNode::Address(adr) => adr.leaves(),
            VcardValueNode::Kind(kind) => vec![kind.leaf()],
            VcardValueNode::Leaf(leaf) => vec![leaf],
        }
    }
}
