//! # Values (syntax side)
//!
//! The raw value of a content line, the cursor that edits it in place, and the
//! per-value-type [`Codec`](crate::tree::codec::Codec) impls, one
//! module each, mirroring the model's `value/`.
//!
//! [`VcardValueNode`] is the generic, byte-faithful value (`;`-separated
//! components of `,`-separated leaves); [`VcardValueCursor`] borrows a line and
//! reads and writes that value through the codec, escaping on write and
//! preserving every component it does not touch. What the components *mean* is
//! the lens's business (see [`crate::tree::prop`]); the codec trait and its
//! structural dispatch live in [`crate::tree::codec`].

mod cursor;
mod node;

mod adr;
mod binary;
mod client_pid_map;
mod datetime;
mod gender;
mod geo;
mod language;
mod n;
mod org;
mod text;
mod unknown;
mod uri;
mod utc_offset;

#[doc(inline)]
pub use cursor::*;
#[doc(inline)]
pub use node::*;
