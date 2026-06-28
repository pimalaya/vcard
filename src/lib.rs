//! vCard (RFC 6350) model, parser and builder.
//!
//! [`rfc6350`] holds the typed model. The `parser` feature adds
//! [`parser::VcardTree`], an edition-ready tree that keeps each property as
//! leaves borrowing the source (or owning edits),
//! so a card rebuilds byte-for-byte or with precise edits. The `builder`
//! feature adds the model's `Display` / to_string encoding. Both features are
//! on by default, and the model alone needs neither.
//!
//! The structured value types (`VcardName`, `VcardAddress`, ...) and the
//! extension RFC modules are decode targets for typed access. Early draft:
//! only the N property is decomposed into typed components; every other property
//! keeps its value as one raw leaf; there is no line unfolding yet.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

pub mod rfc6350;
pub mod rfc6474;
pub mod rfc6715;
pub mod rfc8605;
pub mod rfc9554;
pub mod rfc9555;

#[cfg(feature = "builder")]
mod builder;
#[cfg(feature = "parser")]
pub mod error;
#[cfg(feature = "parser")]
pub mod parser;
