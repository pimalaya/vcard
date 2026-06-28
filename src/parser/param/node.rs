use core::fmt::{self, Display, Formatter};

use alloc::vec::Vec;

use crate::parser::{leaf::VcardLeaf, utils::param_values};

/// One parameter: its name leaf and its `,`-separated value leaves. The leading
/// `;` and the `=` are emitted on serialize, not stored.
pub struct VcardParamNode<'a> {
    /// The parameter name leaf, for example TYPE or PID.
    pub name: VcardLeaf<'a>,
    /// The value leaves, one per comma-separated value; empty when the
    /// parameter carries no `=` value list.
    pub values: Vec<VcardLeaf<'a>>,
}

impl<'a> VcardParamNode<'a> {
    pub(crate) fn parse(param: &'a str) -> Self {
        match param.split_once('=') {
            Some((name, values)) => Self {
                name: VcardLeaf::from(name),
                values: param_values(values),
            },
            None => Self {
                name: VcardLeaf::from(param),
                values: Vec::new(),
            },
        }
    }
}

impl Display for VcardParamNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.text())?;

        if let Some((first, rest)) = self.values.split_first() {
            write!(f, "={}", first.text())?;
            for value in rest {
                write!(f, ",{}", value.text())?;
            }
        }

        Ok(())
    }
}
