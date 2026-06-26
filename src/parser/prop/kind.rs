//! The KIND property: its marker type, parsed value and decoding.

use core::ops::Range;

use crate::{
    parser::{
        decode::VcardDecode, leaf::VcardLeaf, prop::lens::VcardPropLens, utils::unescape,
        value::VcardValueNode,
    },
    rfc6350::prop::kind::{KIND, VcardKind},
};

/// The KIND property as a type, for type-driven lookups (`card.prop::<KIND>()`).
pub struct KIND {}

/// The KIND property as its single value leaf.
#[derive(Clone, Debug)]
pub struct VcardKindNode {
    /// The kind value leaf.
    pub value: VcardLeaf,
}

impl VcardKindNode {
    pub(crate) fn parse(value: Range<usize>) -> Self {
        Self {
            value: VcardLeaf::new(value),
        }
    }

    pub(crate) fn leaf(&self) -> &VcardLeaf {
        &self.value
    }
}

impl VcardPropLens for KIND {
    const NAME: &'static str = KIND;

    type Target = VcardKindNode;

    fn get(value: &VcardValueNode) -> Option<&Self::Target> {
        match value {
            VcardValueNode::Kind(kind) => Some(kind),
            _ => None,
        }
    }

    fn get_mut(value: &mut VcardValueNode) -> Option<&mut Self::Target> {
        match value {
            VcardValueNode::Kind(kind) => Some(kind),
            _ => None,
        }
    }
}

impl<'a> VcardDecode<'a> for VcardKindNode {
    type Output = VcardKind<'a>;

    fn decode(&'a self, input: &'a str) -> VcardKind<'a> {
        let text = self.value.text(input);

        if text.eq_ignore_ascii_case("individual") {
            VcardKind::Individual
        } else if text.eq_ignore_ascii_case("group") {
            VcardKind::Group
        } else if text.eq_ignore_ascii_case("org") {
            VcardKind::Org
        } else if text.eq_ignore_ascii_case("location") {
            VcardKind::Location
        } else {
            VcardKind::Other(unescape(text))
        }
    }
}
