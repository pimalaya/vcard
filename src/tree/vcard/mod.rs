//! # Whole-card strict layer
//!
//! The "strict out" half of the crate, both facing the decoded card: the
//! [`builder`] constructs one property at a time against its spec, and
//! [`validate`] checks a whole [`Vcard`](crate::vcard::Vcard) against the RFC
//! 6350 rules and mints a [`Valid`](validate::Valid) proof. Both share the
//! same per-property check, and both live here so the tree's read side
//! ([`cst`](crate::tree::cst), [`codec`](crate::tree::codec)) and its write
//! side stay visibly separate.

pub mod builder;
pub mod validate;
