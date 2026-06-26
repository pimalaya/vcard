use crate::parser::value::VcardValueNode;

/// A property identified by type: its wire name and the value type it parses
/// into, so it can be looked up with [`VcardTree`](crate::parser::VcardTree) with `prop` / `prop_mut`.
pub trait VcardPropLens {
    /// The property name on the wire.
    const NAME: &'static str;

    /// The structured value this property parses into.
    type Target;

    /// Borrow the typed value out of a generic parsed value, when it matches.
    fn get(value: &VcardValueNode) -> Option<&Self::Target>;

    /// Borrow the typed value out of a generic parsed value mutably.
    fn get_mut(value: &mut VcardValueNode) -> Option<&mut Self::Target>;
}
