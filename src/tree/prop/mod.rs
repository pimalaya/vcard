//! # Property lenses
//!
//! One module per RFC 6350 property, each a hand-written lens marker (and a
//! custom edit cursor where the value is structured).
//!
//! A marker is a zero-sized type tying the wire name to its decoded value type
//! and the `decode`/`encode` projections; it is the type-level key for
//! [`VcardCst::prop`](crate::tree::cst::VcardCst::prop) and friends. Scalar, list
//! and URI properties share the generic
//! [`VcardValueCursor`](crate::tree::cursor::VcardValueCursor); the structured
//! ones (`N`, `ADR`, `GENDER`, `CLIENTPIDMAP`) carry a cursor that names their
//! components. The name dispatch for whole-card decoding lives in
//! [`crate::tree::decode`].

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
