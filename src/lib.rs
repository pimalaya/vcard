#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # vcard-rs
//!
//! A single, version-agnostic vCard library: one model and one byte-faithful
//! syntax tree that parse 2.1 (versitcard), 3.0 (RFC 2426) and 4.0 (RFC 6350)
//! alike. The card's [`VERSION`](version) is a decoded indicator, not a
//! separate dialect: the syntax tree never consults it, and only the codec
//! branches on it where the escaping or a value's shape genuinely differ.
//!
//! - the decoded model (no syntax dependency): [`vcard`] (the card), [`version`],
//!   [`prop`] (property + names), [`param`] (parameter + names), [`value`] (the
//!   value kinds).
//! - [`tree`] — everything syntactic, gated behind the `parser` feature (on by
//!   default; turn it off to depend on the decoded model alone).

extern crate alloc;

pub mod param;
pub mod prop;
pub mod value;
pub mod vcard;
pub mod version;

#[cfg(feature = "parser")]
pub mod tree;
