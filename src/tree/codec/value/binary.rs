//! # Binary value codec (vCard 2.1 / 3.0)
//!
//! [`Codec`] for a 2.1 / 3.0 `PHOTO` / `LOGO` / `SOUND` / `KEY` value: inline
//! base64 kept verbatim. A URI reference is reached via `VALUE=uri`, which the
//! spec resolves to [`VcardUri`](crate::value::uri::VcardUri), not here.

use crate::{
    tree::{
        codec::{encode::scalar_node, mode::Escaper, value::Codec},
        value::VcardValueNode,
    },
    value::binary::VcardBinary,
};

impl<'v> Codec<'v> for VcardBinary<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        VcardBinary::Base64(node.decode_scalar_at(0))
    }

    fn encode(&self, escaper: Escaper) -> VcardValueNode<'static> {
        let raw = match self {
            VcardBinary::Uri(value) | VcardBinary::Base64(value) => value.as_ref(),
        };

        scalar_node(raw, escaper)
    }
}
