use crate::parser::{
    decode::VcardDecode,
    leaf::VcardLeaf,
    param::{lens::VcardParamLens, node::VcardParamNode},
};

/// A property projected to a known value type, its parts borrowed.
pub struct VcardPropView<'a, V> {
    pub input: &'a str,
    /// The property name leaf.
    pub name: &'a VcardLeaf,
    /// The parameters.
    pub params: &'a [VcardParamNode],
    /// The typed value.
    pub value: &'a V,
}

impl<'a, V> VcardPropView<'a, V> {
    /// The first parameter of type `P` (for example `prop.param::<PID>()`). Takes
    /// `&self` and still returns a borrow tied to the card, because the shared
    /// parameter slice is `Copy` and can be lifted out from behind the borrow.
    pub fn param<L: VcardParamLens>(&self) -> Option<&'a VcardParamNode> {
        L::get(self.input, self.params)
    }
}

impl<'a, V: VcardDecode<'a>> VcardPropView<'a, V> {
    /// Decode the parsed value into its real model type (for example
    /// `VcardNameNode` into `VcardName`), resolving escapes and borrowing the
    /// source where no unescaping is needed.
    pub fn decode(&self) -> V::Output {
        self.value.decode(self.input)
    }
}
