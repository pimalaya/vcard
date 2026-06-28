use alloc::vec::Vec;

use crate::parser::{
    leaf::VcardLeaf,
    param::{lens::VcardParamLens, node::VcardParamNode},
};

/// A [`VcardPropView`](super::view::VcardPropView) with mutable access to every
/// part. The `'t` lifetime is the borrow of the card; `'a` is the source the
/// leaves point at.
pub struct VcardPropViewMut<'t, 'a, T> {
    /// The property name leaf.
    pub name: &'t mut VcardLeaf<'a>,
    /// The parameters.
    pub params: &'t mut Vec<VcardParamNode<'a>>,
    /// The typed value.
    pub value: &'t mut T,
}

impl<'a, T> VcardPropViewMut<'_, 'a, T> {
    /// The first parameter of type `L`, mutably. Returns a borrow tied to the
    /// view (the mutable parameter `Vec` is not `Copy`, so it can only be
    /// reborrowed), so bind the view first rather than chaining off a temporary.
    pub fn param_mut<L: VcardParamLens>(&mut self) -> Option<&mut VcardParamNode<'a>> {
        L::get_mut(self.params)
    }
}
