//! # N value codec (RFC 6350 6.2.2)
//!
//! [`VcardCodec`] for the structured name: five `;`-separated components.

use alloc::vec;

use crate::{
    tree::{
        codec::{VcardCodec, encode::encode_component, mode::VcardEscaper},
        value::node::VcardValueNode,
    },
    value::n::VcardN,
};

impl<'v> VcardCodec<'v> for VcardN<'v> {
    fn decode(node: &'v VcardValueNode<'_>) -> Self {
        let mut lists = node.decode_component_lists();
        let mut next = || lists.next().unwrap_or_default();

        VcardN {
            family: next(),
            given: next(),
            additional: next(),
            prefixes: next(),
            suffixes: next(),
        }
    }

    fn encode(&self, escaper: VcardEscaper) -> VcardValueNode<'static> {
        VcardValueNode::from_components(
            vec![
                encode_component(&self.family, escaper),
                encode_component(&self.given, escaper),
                encode_component(&self.additional, escaper),
                encode_component(&self.prefixes, escaper),
                encode_component(&self.suffixes, escaper),
            ],
            escaper,
        )
    }
}
