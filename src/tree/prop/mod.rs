//! # Property lenses
//!
//! The property lens contract and one module per property the crate knows,
//! the read-and-edit half of the markers whose RFC contract lives in
//! [`crate::prop`].
//!
//! [`VcardPropLens`](lens::VcardPropLens) ties a property to a decoded value
//! type plus the `decode` projection and an edit cursor. Each property
//! implements it on its marker, the type-level key for
//! [`VcardCst::prop`](crate::tree::cst::VcardCst::prop).
//!
//! Scalar, list and URI properties share the generic
//! [`VcardValueCursor`](crate::tree::value::cursor::VcardValueCursor); the
//! structured ones (`N`, `ADR`, `GENDER`, `CLIENTPIDMAP`) carry a cursor
//! naming their components, and the version-specific ones (`GEO`, the binary
//! properties) override `decode`.

pub mod adr;
pub mod agent;
pub mod anniversary;
pub mod bday;
pub mod caladruri;
pub mod caluri;
pub mod categories;
pub mod class;
pub mod client_pid_map;
pub mod created;
pub mod email;
pub mod fburl;
pub mod r#fn;
pub mod gender;
pub mod geo;
pub mod gramgender;
pub mod impp;
pub mod jsprop;
pub mod key;
pub mod kind;
pub mod label;
pub mod lang;
pub mod language;
pub mod lens;
pub mod logo;
pub mod mailer;
pub mod member;
pub mod n;
pub mod name;
pub mod nickname;
pub mod note;
pub mod org;
pub mod photo;
pub mod prodid;
pub mod profile;
pub mod pronouns;
pub mod related;
pub mod rev;
pub mod role;
pub mod socialprofile;
pub mod sort_string;
pub mod sound;
pub mod source;
pub mod tel;
pub mod title;
pub mod tz;
pub mod uid;
pub mod url;
pub mod xml;
