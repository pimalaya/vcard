//! # GEO value codec (RFC 6350 6.5.2)
//!
//! [`Codec`] for the 2.1 / 3.0 coordinate pair. vCard 4.0 carries `GEO` as a
//! URI, so that form never reaches this codec (the spec routes it to
//! [`VcardUri`](crate::value::uri::VcardUri)).

use alloc::vec;

use crate::{
    tree::{
        codec::{encode::encode_component, mode::Escaper, value::Codec},
        value::VcardValueNode,
    },
    value::geo::VcardGeo,
};

impl<'v> Codec<'v> for VcardGeo<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        // NOTE: 2.1 separates the pair with `,` (one component, two values), 3.0
        // with `;` (two components); the node's escaper tells them apart.
        if node.escaper == Escaper::V2_1 {
            let mut parts = node.decode_at(0).into_iter();
            VcardGeo {
                latitude: parts.next().unwrap_or_default(),
                longitude: parts.next().unwrap_or_default(),
            }
        } else {
            VcardGeo {
                latitude: node.decode_scalar_at(0),
                longitude: node.decode_scalar_at(1),
            }
        }
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        // NOTE: mirror of decode: 2.1 writes the pair as `lat,long` (one
        // component, two values), 3.0 as `lat;long` (two components).
        let components = if escaper == Escaper::V2_1 {
            vec![encode_component(
                &[self.latitude.as_ref(), self.longitude.as_ref()],
                escaper,
            )]
        } else {
            vec![
                encode_component(&[self.latitude.as_ref()], escaper),
                encode_component(&[self.longitude.as_ref()], escaper),
            ]
        };

        VcardValueNode {
            escaper,
            components,
        }
    }
}
