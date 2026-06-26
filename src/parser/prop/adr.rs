//! The ADR property: its marker type, parsed value and decoding.

use core::ops::Range;

use alloc::vec::Vec;

use crate::{
    parser::{
        decode::VcardDecode,
        leaf::VcardLeaf,
        prop::lens::VcardPropLens,
        utils::{components, decode_values, value_leaves},
        value::VcardValueNode,
    },
    rfc6350::prop::adr::{ADR, VcardAddress},
};

/// The ADR property as a type, for type-driven lookups (`card.prop::<ADR>()`).
pub struct ADR {}

/// The ADR property as its seven `;`-separated components, each a list of
/// `,`-separated value leaves.
#[derive(Clone, Debug)]
pub struct VcardAddressNode {
    /// Post office box values (deprecated).
    pub po_box: Vec<VcardLeaf>,
    /// Extended address values (deprecated).
    pub extended: Vec<VcardLeaf>,
    /// Street address values.
    pub street: Vec<VcardLeaf>,
    /// Locality (city) values.
    pub locality: Vec<VcardLeaf>,
    /// Region (state or province) values.
    pub region: Vec<VcardLeaf>,
    /// Postal code values.
    pub postal_code: Vec<VcardLeaf>,
    /// Country name values.
    pub country: Vec<VcardLeaf>,
}

impl VcardAddressNode {
    pub(crate) fn parse(input: &str, value: Range<usize>) -> Self {
        let [
            po_box,
            extended,
            street,
            locality,
            region,
            postal_code,
            country,
        ] = components::<7>(input, value).map(|component| value_leaves(input, component));

        Self {
            po_box,
            extended,
            street,
            locality,
            region,
            postal_code,
            country,
        }
    }

    pub(crate) fn leaves(&self) -> Vec<&VcardLeaf> {
        [
            &self.po_box,
            &self.extended,
            &self.street,
            &self.locality,
            &self.region,
            &self.postal_code,
            &self.country,
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl VcardPropLens for ADR {
    const NAME: &'static str = ADR;

    type Target = VcardAddressNode;

    fn get(value: &VcardValueNode) -> Option<&Self::Target> {
        match value {
            VcardValueNode::Address(adr) => Some(adr),
            _ => None,
        }
    }

    fn get_mut(value: &mut VcardValueNode) -> Option<&mut Self::Target> {
        match value {
            VcardValueNode::Address(adr) => Some(adr),
            _ => None,
        }
    }
}

impl<'a> VcardDecode<'a> for VcardAddressNode {
    type Output = VcardAddress<'a>;

    fn decode(&'a self, input: &'a str) -> VcardAddress<'a> {
        VcardAddress {
            po_box: decode_values(&self.po_box, input),
            extended: decode_values(&self.extended, input),
            street: decode_values(&self.street, input),
            locality: decode_values(&self.locality, input),
            region: decode_values(&self.region, input),
            postal_code: decode_values(&self.postal_code, input),
            country: decode_values(&self.country, input),
        }
    }
}
