//! # Syntax tree
//!
//! Everything syntactic: the byte-faithful representation of a card and the
//! bridges to and from the decoded model.
//!
//! The hub is [`cst::VcardCst`], a tree of generic nodes ([`line`](mod@line),
//! [`param`], [`value`], [`leaf`]) that round-trips the wire bytes exactly. On
//! top of the generic tree sit the per-name lens markers, each carrying the
//! `VcardPropLens` / `VcardParamLens` contract (plus the per-property
//! `VcardPropSpec`) defined in [`prop`] / [`param`], the in-place edit
//! [`cursor`], the [`codec`] that projects between the tree and the decoded
//! model (decode / encode plus the value escaping), and the spec-driven
//! [`builder`] for strict construction. Parsing is the only fallible step, so
//! its [`error`] type lives here too. This whole layer is gated behind the
//! `parser` feature, so the decoded model can be depended on without it.

pub mod builder;
pub mod codec;
pub mod cst;
pub mod cursor;
pub mod error;
pub mod leaf;
pub mod line;
pub mod param;
pub mod prop;
pub mod validate;
pub mod value;
