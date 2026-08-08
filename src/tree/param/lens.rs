//! # Parameter lens contract
//!
//! [`VcardParamLens`] ties a wire name to a parameter's decoded shape (a single
//! value or a list); the per-name markers in the sibling modules implement it,
//! and it is the type-level key for
//! [`VcardLine::param`](crate::tree::line::VcardLine::param).

use crate::{param::VcardParamKind, tree::param::node::VcardParamNode};

/// A parameter identified by type, projecting a generic syntax parameter onto a
/// decoded value type and back.
pub trait VcardParamLens {
    /// The parameter kind to look up by (its wire name comes through `Deref`).
    const KIND: VcardParamKind;

    /// The decoded value type, borrowing the syntax node for reads.
    type Target<'v>;

    /// Project the generic syntax parameter onto the decoded type.
    fn decode<'v>(param: &'v VcardParamNode<'_>) -> Self::Target<'v>;

    /// Encode a decoded value back into a generic syntax parameter (owned).
    fn encode(decoded: &Self::Target<'_>) -> VcardParamNode<'static>;
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec};

    use crate::tree::param::{
        language::LANGUAGE, lens::VcardParamLens, node::VcardParamNode, pid::PID,
    };

    #[test]
    fn decodes_a_list_parameter_through_its_lens() {
        let node = VcardParamNode::parse("PID=1,2");
        assert_eq!(
            PID::decode(&node),
            vec![Cow::Borrowed("1"), Cow::Borrowed("2")],
        );
    }

    #[test]
    fn encodes_a_scalar_parameter_through_its_lens() {
        let node = LANGUAGE::encode(&Cow::Borrowed("en"));
        assert_eq!(node.to_string(), "LANGUAGE=en");
    }
}
