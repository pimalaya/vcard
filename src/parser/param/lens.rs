use crate::parser::param::node::VcardParamNode;

/// A parameter identified by type, lensing the first match out of a property's
/// parameter list. Parameter values are untyped, so every match is the same
/// [`VcardParamNode`] and the focusing is shared: `get`/`get_mut` are provided.
/// Looked up with
/// [`VcardPropView::param`](crate::parser::prop::view::VcardPropView) and
/// [`VcardPropViewMut::param_mut`](crate::parser::prop::view_mut::VcardPropViewMut).
pub trait VcardParamLens {
    /// The parameter name on the wire.
    const NAME: &'static str;

    /// The first parameter of this type in `params`, if present.
    fn get<'t, 'a>(params: &'t [VcardParamNode<'a>]) -> Option<&'t VcardParamNode<'a>> {
        params
            .iter()
            .find(|param| param.name.text().eq_ignore_ascii_case(Self::NAME))
    }

    /// The first parameter of this type in `params`, mutably.
    fn get_mut<'t, 'a>(params: &'t mut [VcardParamNode<'a>]) -> Option<&'t mut VcardParamNode<'a>> {
        params
            .iter_mut()
            .find(|param| param.name.text().eq_ignore_ascii_case(Self::NAME))
    }
}
