use crate::parser::value::VcardValueNode;

/// A property identified by type: its wire name and the value type it parses
/// into, so it can be looked up on [`VcardTree`](crate::parser::VcardTree) with
/// `prop` / `prop_mut`.
pub trait VcardPropLens {
    /// The property name on the wire.
    const NAME: &'static str;

    /// The structured value this property parses into, borrowing the leaves.
    type Target<'a>;

    /// Borrow the typed value out of a generic parsed value, when it matches.
    fn get<'t, 'a>(value: &'t VcardValueNode<'a>) -> Option<&'t Self::Target<'a>>;

    /// Borrow the typed value out of a generic parsed value mutably.
    fn get_mut<'t, 'a>(value: &'t mut VcardValueNode<'a>) -> Option<&'t mut Self::Target<'a>>;
}
