//! # Property lens contract
//!
//! [`VcardPropLens`] ties a property (by type) to its decoded value type, an
//! edit cursor, and the `decode` projection from the generic syntax node onto
//! the type.
//!
//! The wire name comes from its [`VcardPropSpec::KIND`] supertrait, so the two
//! stay in sync; each property implements it on the marker in its own module.

use crate::{
    prop::spec::VcardPropSpec,
    tree::{codec::VcardCodec, line::VcardLine},
    version::VcardVersion,
};

/// A property identified by type: its decoded value type, edit cursor, and the
/// `decode` projection from the generic syntax node onto the type. The wire
/// name comes from its [`VcardPropSpec::KIND`] (a supertrait), so the two stay
/// in sync.
pub trait VcardPropLens: VcardPropSpec {
    /// The decoded value type, borrowing the syntax node for reads. Its
    /// [`VcardCodec`] impl is what the default `decode` delegates to.
    type Target<'v>: VcardCodec<'v>;

    /// The typed edit cursor over a content line.
    type Cursor<'c, 'a>
    where
        'a: 'c;

    /// Project a content line onto the decoded type.
    ///
    /// The default ignores the version and decodes the node through the
    /// target's [`VcardCodec`]. The lenses whose value shape is
    /// version-specific (`GEO`, the binary props) override it.
    fn decode<'v>(line: &'v VcardLine<'_>, _version: VcardVersion) -> Self::Target<'v> {
        <Self::Target<'v> as VcardCodec<'v>>::decode(&line.value)
    }

    /// Wrap a content line in the typed cursor for in-place editing.
    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> Self::Cursor<'c, 'a>;
}
