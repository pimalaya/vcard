//! # Whole-card strict layer
//!
//! The "strict out" half of the crate, both parts facing the decoded card: the
//! [`builder`] constructs one property at a time against its spec, and
//! [`validate`] checks a whole [`Vcard`](crate::vcard::Vcard) against the RFC
//! 6350 rules and mints a [`VcardValid`](validate::VcardValid) proof.
//!
//! Both share the same per-property check, and both live here so the tree's
//! read side and its write side stay visibly separate.

pub mod builder;
pub mod validate;
