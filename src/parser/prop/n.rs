//! The N property: its marker type, parsed value and decoding.

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
    rfc6350::prop::n::{N, VcardName},
};

/// The N property as a type, for type-driven lookups (`card.prop::<N>()`). It
/// shares its name with the [`N`](crate::rfc6350::prop::n::N) const: the
/// empty-braces struct lives in the type namespace, the const in the value
/// namespace, so both answer to `N`.
pub struct N {}

/// The N property as its five `;`-separated components, each a list of
/// `,`-separated value leaves.
#[derive(Clone, Debug)]
pub struct VcardNameNode<'a> {
    /// Family name values.
    pub family: Vec<VcardLeaf<'a>>,
    /// Given name values.
    pub given: Vec<VcardLeaf<'a>>,
    /// Additional name values.
    pub additional: Vec<VcardLeaf<'a>>,
    /// Honorific prefix values.
    pub prefixes: Vec<VcardLeaf<'a>>,
    /// Honorific suffix values.
    pub suffixes: Vec<VcardLeaf<'a>>,
}

impl<'a> VcardNameNode<'a> {
    pub(crate) fn parse(value: &'a str) -> Self {
        let [family, given, additional, prefixes, suffixes] =
            components::<5>(value).map(value_leaves);

        Self {
            family,
            given,
            additional,
            prefixes,
            suffixes,
        }
    }
}

impl Display for VcardNameNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_components(
            f,
            &[
                &self.family,
                &self.given,
                &self.additional,
                &self.prefixes,
                &self.suffixes,
            ],
        )
    }
}

impl VcardPropLens for N {
    const NAME: &'static str = N;

    type Target<'a> = VcardNameNode<'a>;

    fn get<'t, 'a>(value: &'t VcardValueNode<'a>) -> Option<&'t VcardNameNode<'a>> {
        match value {
            VcardValueNode::Name(name) => Some(name),
            _ => None,
        }
    }

    fn get_mut<'t, 'a>(value: &'t mut VcardValueNode<'a>) -> Option<&'t mut VcardNameNode<'a>> {
        match value {
            VcardValueNode::Name(name) => Some(name),
            _ => None,
        }
    }
}

impl VcardDecode for VcardNameNode<'_> {
    type Output<'o>
        = VcardName<'o>
    where
        Self: 'o;

    fn decode(&self) -> VcardName<'_> {
        VcardName {
            family: decode_values(&self.family),
            given: decode_values(&self.given),
            additional: decode_values(&self.additional),
            prefixes: decode_values(&self.prefixes),
            suffixes: decode_values(&self.suffixes),
        }
    }
}
