//! # GEO value codec (RFC 6350 6.5.2)
//!
//! [`VcardCodec`] for the 2.1 / 3.0 coordinate pair. vCard 4.0 carries `GEO` as
//! a URI, so that form never reaches this codec (the spec routes it to
//! [`VcardUri`](crate::value::uri::VcardUri)).

use crate::{
    tree::{
        codec::{VcardCodec, encode::VcardValueOut, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::geo::VcardGeo,
};

impl<'v> VcardCodec<'v> for VcardGeo<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        // NOTE: 2.1 separates the pair with `,` (one component, two values),
        // 3.0 with `;` (two components); the node's escaper tells them apart.
        if node.escaper == VcardEscaper::V2_1 {
            let mut parts = node.decode_component_list(0).into_iter();
            VcardGeo {
                latitude: parts.next().unwrap_or_default(),
                longitude: parts.next().unwrap_or_default(),
            }
        } else {
            VcardGeo {
                latitude: node.decode_component(0),
                longitude: node.decode_component(1),
            }
        }
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueOut::node(escaper, |out| self.emit(out))
    }
}

impl VcardGeo<'_> {
    /// Write the pair in the card's own version.
    pub(crate) fn emit(&self, out: &mut VcardValueOut<'_>) {
        // NOTE: mirror of decode: 2.1 writes the pair as `lat,long` (one
        // component, two values), 3.0 as `lat;long` (two components).
        if out.escaper == VcardEscaper::V2_1 {
            out.component(&[&*self.latitude, &*self.longitude]);
        } else {
            out.component(&[&*self.latitude]);
            out.component(&[&*self.longitude]);
        }
    }
}
