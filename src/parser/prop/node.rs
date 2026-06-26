use alloc::vec::Vec;

use crate::{
    parser::{
        leaf::VcardLeaf,
        line::VcardLine,
        param::node::VcardParamNode,
        prop::{adr::VcardAddressNode, kind::VcardKindNode, n::VcardNameNode},
        utils::{parse_params, split_name_params},
        value::VcardValueNode,
    },
    rfc6350::prop::{adr::ADR, kind::KIND, n::N},
};

/// One parsed property: its name leaf, its parameters and its value.
pub struct VcardPropNode {
    /// The property name leaf, with any group prefix.
    pub name: VcardLeaf,
    /// The parameters, in source order; any parameter may appear.
    pub params: Vec<VcardParamNode>,
    /// The value, decomposed when the property is modelled.
    pub value: VcardValueNode,
}

impl VcardPropNode {
    pub(crate) fn parse(input: &str, line: &VcardLine) -> Self {
        let (name_range, params_range) = split_name_params(input, line.prop.clone());
        let value_range = line.value.clone();
        let name = &input[name_range.clone()];

        let value = if name.eq_ignore_ascii_case(N) {
            VcardValueNode::Name(VcardNameNode::parse(input, value_range))
        } else if name.eq_ignore_ascii_case(ADR) {
            VcardValueNode::Address(VcardAddressNode::parse(input, value_range))
        } else if name.eq_ignore_ascii_case(KIND) {
            VcardValueNode::Kind(VcardKindNode::parse(value_range))
        } else {
            VcardValueNode::Leaf(VcardLeaf::new(value_range))
        };

        Self {
            name: VcardLeaf::new(name_range),
            params: parse_params(input, params_range),
            value,
        }
    }
}
