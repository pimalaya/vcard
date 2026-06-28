//! The KIND property: its marker type, parsed value and decoding.

use core::fmt::{self, Display, Formatter};

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
pub struct VcardKindNode<'a> {
    /// The kind value leaf.
    pub value: VcardLeaf<'a>,
}

impl<'a> VcardKindNode<'a> {
    pub(crate) fn parse(value: &'a str) -> Self {
        Self {
            value: VcardLeaf::from(value),
        }
    }
}

impl Display for VcardKindNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.value.text())
    }
}

impl VcardPropLens for KIND {
    const NAME: &'static str = KIND;

    type Target<'a> = VcardKindNode<'a>;

    fn get<'t, 'a>(value: &'t VcardValueNode<'a>) -> Option<&'t VcardKindNode<'a>> {
        match value {
            VcardValueNode::Kind(kind) => Some(kind),
            _ => None,
        }
    }

    fn get_mut<'t, 'a>(value: &'t mut VcardValueNode<'a>) -> Option<&'t mut VcardKindNode<'a>> {
        match value {
            VcardValueNode::Kind(kind) => Some(kind),
            _ => None,
        }
    }
}

impl VcardDecode for VcardKindNode<'_> {
    type Output<'o>
        = VcardKind<'o>
    where
        Self: 'o;

    fn decode(&self) -> VcardKind<'_> {
        let text = self.value.text();

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
