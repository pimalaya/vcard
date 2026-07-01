//! # Property lens contract
//!
//! [`VcardPropLens`] ties a property (by type) to its decoded value type, an
//! edit cursor, and the `decode` projection from the generic syntax node onto
//! the type. The wire name comes from its [`VcardPropSpec::KIND`] supertrait, so
//! the two stay in sync; each property implements it on the marker in its own
//! module.

use crate::{
    tree::{codec::Codec, line::VcardLine, prop::VcardPropSpec},
    version::VcardVersion,
};

/// A property identified by type: its decoded value type, edit cursor, and the
/// `decode` projection from the generic syntax node onto the type. The wire name
/// comes from its [`VcardPropSpec::KIND`] (a supertrait), so the two stay in
/// sync.
pub trait VcardPropLens: VcardPropSpec {
    /// The decoded value type, borrowing the syntax node for reads. Its
    /// [`Codec`] impl is what the default `decode` delegates to.
    type Target<'v>: Codec<'v>;

    /// The typed edit cursor over a content line.
    type Cursor<'c, 'a>
    where
        'a: 'c;

    /// Project a content line onto the decoded type, consulting the card version
    /// where the value's shape is version-specific (`GEO`, the binary props).
    /// The default ignores the version and decodes the value node through the
    /// target's [`Codec`]; the version-specific lenses override it.
    fn decode<'v>(line: &'v VcardLine<'_>, _version: VcardVersion) -> Self::Target<'v> {
        <Self::Target<'v> as Codec<'v>>::decode(&line.value)
    }

    /// Wrap a content line in the typed cursor for in-place editing.
    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> Self::Cursor<'c, 'a>;
}
