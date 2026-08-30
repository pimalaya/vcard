//! # Syntax tree
//!
//! Everything syntactic: the byte-faithful representation of a card and the
//! bridges to and from the decoded model.
//!
//! The hub is [`cst::VcardCst`], a tree of generic nodes ([`line`](mod@line),
//! [`param`], [`value`], [`leaf`]) round-tripping the wire bytes exactly, each
//! line keeping the [`wire`] shape it arrived in.
//!
//! On top of it sit the per-name lens markers in [`prop`] and [`param`], each
//! carrying its lens contract, plus the in-place edit cursor in [`value`]. A
//! property marker is defined in [`crate::prop`] with the RFC contract it also
//! carries; only its lens half lives here.
//!
//! The [`codec`] projects between tree and decoded model, and the three-way
//! [`merge`](mod@merge) reconciles two divergent edits against their common
//! base. The strict way out, [`crate::builder`] and [`crate::validator`], is
//! model rather than syntax and sits at the crate root.
//!
//! Parsing is the only fallible step, so its [`error`] type lives here too.
//! This whole layer is gated behind the `parser` feature, so the decoded model
//! can be depended on without it.

pub mod codec;
pub mod cst;
pub mod error;
pub mod leaf;
pub mod line;
pub mod merge;
pub mod param;
pub mod prop;
pub mod value;
pub mod wire;
