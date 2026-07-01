//! # ADR value codec (RFC 6350 6.3.1)
//!
//! [`Codec`] for the structured address: seven `;`-separated components.

use alloc::vec;

use crate::{
    tree::{
        codec::{encode::encode_component, mode::Escaper, value::Codec},
        value::VcardValueNode,
    },
    value::adr::VcardAdr,
};

impl<'v> Codec<'v> for VcardAdr<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardAdr {
            po_box: node.decode_at(0),
            extended: node.decode_at(1),
            street: node.decode_at(2),
            locality: node.decode_at(3),
            region: node.decode_at(4),
            postal_code: node.decode_at(5),
            country: node.decode_at(6),
        }
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        VcardValueNode {
            escaper,
            components: vec![
                encode_component(&self.po_box, escaper),
                encode_component(&self.extended, escaper),
                encode_component(&self.street, escaper),
                encode_component(&self.locality, escaper),
                encode_component(&self.region, escaper),
                encode_component(&self.postal_code, escaper),
                encode_component(&self.country, escaper),
            ],
        }
    }
}
