#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # vcard-rs
//!
//! A vCard library, organised one module per spec version behind its own feature
//! gate. Each version module is self-contained (its own decoded model, syntax
//! tree, and codec); shared pieces will be factored out once more than one
//! version exists.
//!
//! - [`v4_0`] — vCard 4.0 (RFC 6350). Enabled by the `v4_0` feature (default).

extern crate alloc;

#[cfg(feature = "v4_0")]
pub mod v4_0;
