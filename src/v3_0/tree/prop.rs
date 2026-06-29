//! # Property lenses
//!
//! The property lens contract and one hand-written module per RFC 2426 property.
//!
//! [`VcardPropLens`] ties a wire name to a decoded value type plus the
//! `decode`/`encode` projections and an edit cursor; each property implements it
//! in its own submodule, where the marker is the type-level key for
//! [`VcardCst::prop`](crate::v3_0::tree::cst::VcardCst::prop). Scalar, list, URI
//! and binary properties share the generic
//! [`VcardValueCursor`](crate::v3_0::tree::cursor::VcardValueCursor); the structured
//! ones (`N`, `ADR`, `GEO`) carry a cursor that names their components. The name
//! dispatch for whole-card decoding lives in [`crate::v3_0::tree::decode`].

pub mod adr;
pub mod agent;
pub mod bday;
pub mod categories;
pub mod class;
pub mod email;
pub mod r#fn;
pub mod geo;
pub mod key;
pub mod label;
pub mod logo;
pub mod mailer;
pub mod n;
pub mod name;
pub mod nickname;
pub mod note;
pub mod org;
pub mod photo;
pub mod prodid;
pub mod profile;
pub mod rev;
pub mod role;
pub mod sort_string;
pub mod sound;
pub mod source;
pub mod tel;
pub mod title;
pub mod tz;
pub mod uid;
pub mod url;

use crate::v3_0::tree::{line::VcardLine, value::VcardValueNode};

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
