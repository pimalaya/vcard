//! # Lens contract
//!
//! The traits that pin a wire name to a decoded type and project between them.
//!
//! A lens is a zero-sized marker type (e.g. [`N`](crate::tree::prop::n::N)) that
//! ties a property or parameter name to the decoded value it produces. The
//! marker carries the wire [`NAME`](VcardPropLens::NAME), the decoded
//! [`Target`](VcardPropLens::Target) type, and the `decode`/`encode` projections
//! between the generic syntax node and that type; properties additionally carry
//! an edit [`Cursor`](VcardPropLens::Cursor). All per-name semantics live in the
//! per-name lens modules ([`crate::tree::prop`], [`crate::tree::param`]), so the
//! rest of [`crate::tree`] stays fully generic over the syntax, and the decoded
//! model stays free of any name dispatch.

use crate::tree::{line::VcardLine, param::VcardParamNode, value::VcardValueNode};

/// A property identified by type: its wire name, decoded value type, edit
/// cursor, and the projections between the generic syntax node and the type.
pub trait VcardPropLens {
    /// The wire name to look up by.
    const NAME: &'static str;

    /// The decoded value type, borrowing the syntax node for reads.
    type Target<'v>;

    /// The typed edit cursor over a content line.
    type Cursor<'c, 'a>
    where
        'a: 'c;

    /// Project the generic syntax node onto the decoded type (unescaping).
    fn decode<'v>(value: &'v VcardValueNode<'_>) -> Self::Target<'v>;

    /// Encode a decoded value back into a generic syntax node (escaping, owned).
    fn encode(decoded: &Self::Target<'_>) -> VcardValueNode<'static>;

    /// Wrap a content line in the typed cursor for in-place editing.
    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> Self::Cursor<'c, 'a>;
}

/// A parameter identified by type, projecting a generic syntax parameter onto a
/// decoded value type and back.
pub trait VcardParamLens {
    /// The wire name to look up by.
    const NAME: &'static str;

    /// The decoded value type, borrowing the syntax node for reads.
    type Target<'v>;

    /// Project the generic syntax parameter onto the decoded type.
    fn decode<'v>(param: &'v VcardParamNode<'_>) -> Self::Target<'v>;

    /// Encode a decoded value back into a generic syntax parameter (owned).
    fn encode(decoded: &Self::Target<'_>) -> VcardParamNode<'static>;
}
