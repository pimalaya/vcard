use alloc::vec::Vec;

use crate::parser::{
    leaf::VcardLeaf,
    param::{lens::VcardParamLens, node::VcardParamNode},
};

/// A [`VcardPropView`](super::view::VcardPropView) with mutable access to every
/// part.
pub struct VcardPropViewMut<'a, V> {
    pub input: &'a str,
    /// The property name leaf.
    pub name: &'a mut VcardLeaf,
    /// The parameters.
    pub params: &'a mut Vec<VcardParamNode>,
    /// The typed value.
    pub value: &'a mut V,
}

impl<'a, V> VcardPropViewMut<'a, V> {
    /// The first parameter of type `L`, mutably. Returns a borrow tied to the
    /// view (the mutable parameter `Vec` is not `Copy`, so it can only be
    /// reborrowed), so bind the view first rather than chaining off a temporary.
    pub fn param_mut<L: VcardParamLens>(&mut self) -> Option<&mut VcardParamNode> {
        L::get_mut(self.input, self.params)
    }
}
