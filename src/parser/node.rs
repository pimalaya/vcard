use alloc::vec::Vec;

use crate::parser::{
    leaf::VcardLeaf, param::node::VcardParamNode, prop::node::VcardPropNode, value::VcardValueNode,
};

/// A depth-first iterator over every leaf in a card. It keeps its own worklist
/// on the heap instead of recursing, so a deeply nested tree never grows the
/// call stack.
pub struct VcardNodes<'a> {
    pub stack: Vec<VcardNode<'a>>,
}

/// A pending node in the [`VcardNodes`] walk: either a leaf to yield or a
/// parent to expand into its children.
pub enum VcardNode<'a> {
    Property(&'a VcardPropNode),
    Parameter(&'a VcardParamNode),
    Value(&'a VcardValueNode),
    Leaf(&'a VcardLeaf),
}

impl<'a> Iterator for VcardNodes<'a> {
    type Item = &'a VcardLeaf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            match node {
                VcardNode::Leaf(leaf) => return Some(leaf),
                VcardNode::Property(property) => {
                    // pushed in reverse so they pop back in source order
                    self.stack.push(VcardNode::Value(&property.value));
                    for param in property.params.iter().rev() {
                        self.stack.push(VcardNode::Parameter(param));
                    }
                    self.stack.push(VcardNode::Leaf(&property.name));
                }
                VcardNode::Parameter(param) => {
                    for value in param.values.iter().rev() {
                        self.stack.push(VcardNode::Leaf(value));
                    }
                    self.stack.push(VcardNode::Leaf(&param.name));
                }
                VcardNode::Value(value) => {
                    for leaf in value.leaves().into_iter().rev() {
                        self.stack.push(VcardNode::Leaf(leaf));
                    }
                }
            }
        }

        None
    }
}
