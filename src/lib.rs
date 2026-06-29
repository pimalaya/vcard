#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # vcard-rs
//!
//! A vCard library, organised one module per spec version behind its own feature
//! gate. Each version module is self-contained (its own decoded model, syntax
//! tree, and codec); shared pieces will be factored out once more than one
//! version exists.
//!
//! - [`v40`] — vCard 4.0 (RFC 6350). Enabled by the `v40` feature (default).
//! - [`v30`] — vCard 3.0 (RFC 2426). Enabled by the `v30` feature (default).
//! - [`v21`] — vCard 2.1 (versitcard). Enabled by the `v21` feature.

extern crate alloc;

#[cfg(feature = "v21")]
#[path = "2.1/mod.rs"]
pub mod v21;
#[cfg(feature = "v30")]
#[path = "3.0/mod.rs"]
pub mod v30;
#[cfg(feature = "v40")]
#[path = "4.0/mod.rs"]
pub mod v40;
