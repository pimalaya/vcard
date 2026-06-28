//! The ADR property: its marker type, parsed value and decoding.

use core::fmt::{self, Display, Formatter};

use alloc::vec::Vec;

use crate::{
    parser::{
        decode::VcardDecode,
        leaf::VcardLeaf,
        prop::lens::VcardPropLens,
        utils::{components, decode_values, value_leaves, write_components},
        value::VcardValueNode,
    },
    rfc6350::prop::adr::{ADR, VcardAddress},
};

/// The ADR property as a type, for type-driven lookups (`card.prop::<ADR>()`).
pub struct ADR {}

/// The ADR property as its seven `;`-separated components, each a list of
/// `,`-separated value leaves.
#[derive(Clone, Debug)]
pub struct VcardAddressNode<'a> {
    /// Post office box values (deprecated).
    pub po_box: Vec<VcardLeaf<'a>>,
    /// Extended address values (deprecated).
    pub extended: Vec<VcardLeaf<'a>>,
    /// Street address values.
    pub street: Vec<VcardLeaf<'a>>,
    /// Locality (city) values.
    pub locality: Vec<VcardLeaf<'a>>,
    /// Region (state or province) values.
    pub region: Vec<VcardLeaf<'a>>,
    /// Postal code values.
    pub postal_code: Vec<VcardLeaf<'a>>,
    /// Country name values.
    pub country: Vec<VcardLeaf<'a>>,
}

impl<'a> VcardAddressNode<'a> {
    pub(crate) fn parse(value: &'a str) -> Self {
        let [
            po_box,
            extended,
            street,
            locality,
            region,
            postal_code,
            country,
        ] = components::<7>(value).map(value_leaves);

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
}

impl Display for VcardAddressNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_components(
            f,
            &[
                &self.po_box,
                &self.extended,
                &self.street,
                &self.locality,
                &self.region,
                &self.postal_code,
                &self.country,
            ],
        )
    }
}

impl VcardPropLens for ADR {
    const NAME: &'static str = ADR;

    type Target<'a> = VcardAddressNode<'a>;

    fn get<'t, 'a>(value: &'t VcardValueNode<'a>) -> Option<&'t VcardAddressNode<'a>> {
        match value {
            VcardValueNode::Address(adr) => Some(adr),
            _ => None,
        }
    }

    fn get_mut<'t, 'a>(value: &'t mut VcardValueNode<'a>) -> Option<&'t mut VcardAddressNode<'a>> {
        match value {
            VcardValueNode::Address(adr) => Some(adr),
            _ => None,
        }
    }
}

impl VcardDecode for VcardAddressNode<'_> {
    type Output<'o>
        = VcardAddress<'o>
    where
        Self: 'o;

    fn decode(&self) -> VcardAddress<'_> {
        VcardAddress {
            po_box: decode_values(&self.po_box),
            extended: decode_values(&self.extended),
            street: decode_values(&self.street),
            locality: decode_values(&self.locality),
            region: decode_values(&self.region),
            postal_code: decode_values(&self.postal_code),
            country: decode_values(&self.country),
        }
    }
}
