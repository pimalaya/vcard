use core::ops::Range;

use alloc::vec::Vec;

use crate::parser::{leaf::VcardLeaf, utils::split_param_values};

/// One parameter: its name leaf and its `,`-separated value leaves.
pub struct VcardParamNode {
    /// The parameter name leaf, for example TYPE or PID.
    pub name: VcardLeaf,
    /// The value leaves, one per comma-separated value (empty when valueless).
    pub values: Vec<VcardLeaf>,
}

impl VcardParamNode {
    pub(crate) fn parse(input: &str, range: Range<usize>) -> Self {
        match memchr::memchr(b'=', &input.as_bytes()[range.clone()]) {
            Some(rel) => {
                let eq = range.start + rel;
                let values = split_param_values(input, eq + 1..range.end)
                    .into_iter()
                    .map(VcardLeaf::new)
                    .collect();

                Self {
                    name: VcardLeaf::new(range.start..eq),
                    values,
                }
            }
            None => Self {
                name: VcardLeaf::new(range),
                values: Vec::new(),
            },
        }
    }
}
