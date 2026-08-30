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
//! carrying its lens contract and, for a property, its `VcardPropSpec`, plus
//! the in-place edit cursor in [`value`].
//!
//! The [`codec`] projects between tree and decoded model, [`vcard`] is the
//! strict way out (the spec-driven builder and validation), and the three-way
//! [`merge`](mod@merge) reconciles two divergent edits against their common
//! base.
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
pub mod vcard;
pub mod wire;
