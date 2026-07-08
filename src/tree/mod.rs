//! # Syntax tree
//!
//! Everything syntactic: the byte-faithful representation of a card and the
//! bridges to and from the decoded model.
//!
//! The hub is [`cst::VcardCst`], a tree of generic nodes ([`line`](mod@line),
//! [`param`], [`value`], [`leaf`]) that round-trips the wire bytes exactly. On
//! top of the generic tree sit the per-name lens markers, each carrying the
//! `VcardPropLens` / `VcardParamLens` contract (plus the per-property
//! `VcardPropSpec`) defined in [`prop`] / [`param`], the in-place edit cursor
//! in [`value`], the [`codec`] that projects between the tree and the decoded
//! model (decode / encode plus the value escaping), the strict-out layer in
//! [`vcard`] (the spec-driven builder and validation), and the three-way
//! [`merge`](mod@merge) that reconciles two divergent edits of a card against
//! their common base. Parsing is the only fallible step, so its [`error`] type
//! lives here too. This whole layer is gated behind the `parser` feature, so
//! the decoded model can be depended on without it.

pub mod codec;
pub mod cst;
pub mod error;
pub mod leaf;
pub mod line;
pub mod merge;
pub mod param;
pub mod prop;
pub mod value;
pub mod vcard;
