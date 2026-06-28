//! # Syntax tree
//!
//! Everything syntactic: the byte-faithful representation of a card and the
//! bridges to and from the decoded model.
//!
//! The hub is [`cst::VcardCst`], a tree of generic nodes ([`line`](mod@line), [`param`],
//! [`value`], [`leaf`]) that round-trips the wire bytes exactly. On top of the
//! generic tree sit the per-name lens markers ([`prop`], [`param`]) tied together
//! by the [`lens`] contract, the in-place edit [`cursor`], and the
//! [`decode`] / [`encode`] bridges that project between the tree and the decoded
//! model (the crate-root modules) through the value codec. This whole layer is
//! destined to become an optional feature, so the decoded model can be depended
//! on without it.

pub mod cst;
pub mod cursor;
pub mod decode;
pub mod encode;
pub mod leaf;
pub mod lens;
pub mod line;
pub mod param;
pub mod prop;
pub mod value;
