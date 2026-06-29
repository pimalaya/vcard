#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # vcard-rs
//!
//! A vCard library, organised one module per spec version. Each version module
//! carries its own decoded model, syntax tree, and codec; the version-agnostic
//! primitives shared by all of them live at the crate root: [`version`] (the
//! decoded `VERSION` value), [`vcard`] (the `BEGIN:VCARD` / `END:VCARD` framing
//! vocabulary) and [`tree`] (the [`leaf`](tree::leaf) atom and the parse
//! [`error`](tree::error)).
//!
//! - [`v40`] — vCard 4.0 (RFC 6350).
//! - [`v30`] — vCard 3.0 (RFC 2426).
//! - [`v21`] — vCard 2.1 (versitcard).
//!
//! The syntax tree (parsing, serializing, in-place editing) is gated behind the
//! `parser` feature, on by default; turn it off to depend on the decoded model
//! alone.

extern crate alloc;

pub mod vcard;
pub mod version;

#[cfg(feature = "parser")]
pub mod tree;

#[path = "2.1/mod.rs"]
pub mod v21;
#[path = "3.0/mod.rs"]
pub mod v30;
#[path = "4.0/mod.rs"]
pub mod v40;
