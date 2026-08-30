//! # Values (syntax side)
//!
//! The raw value of a content line, the cursor that edits it in place, and the
//! per-value-type [`VcardCodec`](crate::tree::codec::VcardCodec) impls, one
//! module each, mirroring the model's `value/`.
//!
//! [`VcardValueNode`](node::VcardValueNode) is the generic, byte-faithful
//! value: `;`-separated components of `,`-separated leaves.
//!
//! [`VcardValueCursor`](cursor::VcardValueCursor) reads and writes it through
//! the codec, escaping on write and preserving every component it does not
//! touch. What the components *mean* is the lens's business (see
//! [`crate::tree::prop`]).

pub mod cursor;
pub mod node;

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
