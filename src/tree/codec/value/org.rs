//! # ORG value codec (RFC 6350 6.6.4)
//!
//! [`Codec`] for the organization value: one or more `;`-separated units.

use crate::{
    tree::{
        codec::{encode::encode_component, mode::Escaper, value::Codec},
        value::VcardValueNode,
    },
    value::org::VcardOrg,
};

impl<'v> Codec<'v> for VcardOrg<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        let units = (0..node.components.len())
            .map(|i| node.decode_scalar_at(i))
            .collect();
        VcardOrg(units)
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        VcardValueNode {
            escaper,
            components: self
                .0
                .iter()
                .map(|unit| encode_component(&[unit.as_ref()], escaper))
                .collect(),
        }
    }
}
