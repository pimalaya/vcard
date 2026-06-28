use crate::parser::{
    decode::VcardDecode,
    leaf::VcardLeaf,
    param::{lens::VcardParamLens, node::VcardParamNode},
};

/// A property projected to a known value type, its parts borrowed. The `'t`
/// lifetime is the borrow of the card; `'a` is the source the leaves point at.
pub struct VcardPropView<'t, 'a, T> {
    /// The property name leaf.
    pub name: &'t VcardLeaf<'a>,
    /// The parameters.
    pub params: &'t [VcardParamNode<'a>],
    /// The typed value.
    pub value: &'t T,
}

impl<'t, 'a, T> VcardPropView<'t, 'a, T> {
    /// The first parameter of type `L` (for example `prop.param::<PID>()`). Takes
    /// `&self` and still returns a borrow tied to the card, because the shared
    /// parameter slice is `Copy` and can be lifted out from behind the borrow.
    pub fn param<L: VcardParamLens>(&self) -> Option<&'t VcardParamNode<'a>> {
        L::get(self.params)
    }
}

impl<'t, T: VcardDecode> VcardPropView<'t, '_, T> {
    /// Decode the parsed value into its real model type (for example
    /// `VcardNameNode` into `VcardName`), resolving escapes and borrowing the
    /// leaves (which outlive the view) where no unescaping is needed.
    pub fn decode(&self) -> T::Output<'t> {
        self.value.decode()
    }
}
