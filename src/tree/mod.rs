//! # Shared syntax-tree primitives
//!
//! The version-agnostic atoms of the syntax tree, shared by every version
//! module's own `tree`: [`leaf`] (the raw string atom every node is built from)
//! and [`error`] (the single parse error type). Both are pure structure with no
//! property or version semantics, so they live once at the crate root. Gated
//! behind the `parser` feature, like the per-version trees that depend on them.

pub mod error;
pub mod leaf;
