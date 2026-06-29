//! # Property lenses
//!
//! The property lens contract and one hand-written module per RFC 6350 property.
//!
//! [`VcardPropLens`] ties a wire name to a decoded value type plus the
//! `decode`/`encode` projections and an edit cursor; each property implements it
//! in its own submodule, where the marker is the type-level key for
//! [`VcardCst::prop`](crate::v2_1::tree::cst::VcardCst::prop). Scalar, list and URI
//! properties share the generic
//! [`VcardValueCursor`](crate::v2_1::tree::cursor::VcardValueCursor); the structured
//! ones (`N`, `ADR`, `GENDER`, `CLIENTPIDMAP`) carry a cursor that names their
//! components. The name dispatch for whole-card decoding lives in
//! [`crate::v2_1::tree::decode`].

pub mod adr;
pub mod anniversary;
pub mod bday;
pub mod caladruri;
pub mod caluri;
pub mod categories;
pub mod client_pid_map;
pub mod email;
pub mod fburl;
pub mod r#fn;
pub mod gender;
pub mod geo;
pub mod impp;
pub mod key;
pub mod kind;
pub mod lang;
pub mod logo;
pub mod member;
pub mod n;
pub mod nickname;
pub mod note;
pub mod org;
pub mod photo;
pub mod prodid;
pub mod related;
pub mod rev;
pub mod role;
pub mod sound;
pub mod source;
pub mod tel;
pub mod title;
pub mod tz;
pub mod uid;
pub mod url;
pub mod xml;

use crate::v2_1::tree::{line::VcardLine, value::VcardValueNode};

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
