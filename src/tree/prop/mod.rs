//! # Property lenses
//!
//! The property lens contract, the per-property spec, and one hand-written
//! module per RFC 6350 property.
//!
//! [`VcardPropLens`] ties a wire name to a decoded value type plus the `decode`
//! projection and an edit cursor; each property implements it on the marker in
//! its own module, the type-level key for
//! [`VcardCst::prop`](crate::tree::cst::VcardCst::prop). Scalar, list and URI
//! properties share the generic
//! [`VcardValueCursor`](crate::tree::value::VcardValueCursor); the structured
//! ones (`N`, `ADR`, `GENDER`, `CLIENTPIDMAP`) carry a cursor that names their
//! components. The per-property contract is [`VcardPropSpec`], with the
//! [`VcardPropCardinality`] multiplicity axis; the name dispatch for whole-card
//! decoding lives in [`crate::tree::codec::decode`].

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

mod cardinality;
mod lens;
mod spec;

#[doc(inline)]
pub use cardinality::*;
#[doc(inline)]
pub use lens::*;
#[doc(inline)]
pub use spec::VcardPropSpec;

pub(crate) use spec::{VcardPropSpecFns, prop_spec};
