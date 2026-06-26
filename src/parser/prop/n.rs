//! The N property: its marker type, parsed value and decoding.

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
pub struct VcardNameNode {
    /// Family name values.
    pub family: Vec<VcardLeaf>,
    /// Given name values.
    pub given: Vec<VcardLeaf>,
    /// Additional name values.
    pub additional: Vec<VcardLeaf>,
    /// Honorific prefix values.
    pub prefixes: Vec<VcardLeaf>,
    /// Honorific suffix values.
    pub suffixes: Vec<VcardLeaf>,
}

impl VcardNameNode {
    pub(crate) fn parse(input: &str, value: Range<usize>) -> Self {
        let [family, given, additional, prefixes, suffixes] =
            components::<5>(input, value).map(|component| value_leaves(input, component));

        Self {
            family,
            given,
            additional,
            prefixes,
            suffixes,
        }
    }

    pub(crate) fn leaves(&self) -> Vec<&VcardLeaf> {
        [
            &self.family,
            &self.given,
            &self.additional,
            &self.prefixes,
            &self.suffixes,
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl VcardPropLens for N {
    const NAME: &'static str = N;

    type Target = VcardNameNode;

    fn get(value: &VcardValueNode) -> Option<&Self::Target> {
        match value {
            VcardValueNode::Name(name) => Some(name),
            _ => None,
        }
    }

    fn get_mut(value: &mut VcardValueNode) -> Option<&mut Self::Target> {
        match value {
            VcardValueNode::Name(name) => Some(name),
            _ => None,
        }
    }
}

impl<'a> VcardDecode<'a> for VcardNameNode {
    type Output = VcardName<'a>;

    fn decode(&'a self, input: &'a str) -> VcardName<'a> {
        VcardName {
            family: decode_values(&self.family, input),
            given: decode_values(&self.given, input),
            additional: decode_values(&self.additional, input),
            prefixes: decode_values(&self.prefixes, input),
            suffixes: decode_values(&self.suffixes, input),
        }
    }
}
