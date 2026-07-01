#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # vcard-rs
//!
//! A single, version-agnostic vCard library: one decoded model and one
//! byte-faithful syntax tree that read and write vCard 2.1 (versitcard), 3.0
//! (RFC 2426) and 4.0 (RFC 6350) alike. The card version is a decoded
//! indicator, never a type parameter or a separate dialect: the syntax tree
//! ignores it, and only the codec and the per-property spec branch on it where
//! escaping or a value's shape genuinely differ. The crate is `no_std` (with
//! `alloc`) and dependency-free.
//!
//! ## Example
//!
//! Parse raw bytes, read a property through its typed lens, edit it in place,
//! and serialize back; every untouched byte is preserved.
//!
//! ```rust
//! use vcard::tree::cst::VcardCst;
//! use vcard::tree::prop::r#fn::FN;
//!
//! let mut card =
//!     VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n").unwrap();
//!
//! assert_eq!(&*card.prop::<FN>().unwrap().0, "John Doe");
//!
//! card.prop_mut::<FN>().unwrap().set_text("Jane Doe");
//! assert_eq!(
//!     card.to_string(),
//!     "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\nEND:VCARD\r\n",
//! );
//! ```
//!
//! ## Postel's law
//!
//! The library is liberal in what it accepts and strict in what it sends.
//! Parsing is maximally liberal: any real card, including properties,
//! parameters and value types that no version officially defines, is accepted
//! and round-trips byte for byte. The decoded model keeps that openness, with
//! an `Unknown` arm on every open vocabulary. Strictness lives only on the way
//! out, as two runtime steps: the builder, which refuses to construct a
//! property the spec forbids, and [`validate`](tree::vcard::validate), which
//! checks a decoded card against its version's RFC contract.
//!
//! ## The two layers
//!
//! The decoded model ([`vcard`], [`version`], [`prop`], [`param`], [`value`])
//! is pure data with no dependency on the syntax side, so it can be depended on
//! alone. Property names, parameter names and value types are closed identity
//! enums ([`VcardPropKind`](prop::VcardPropKind),
//! [`VcardParamKind`](param::VcardParamKind),
//! [`VcardValueKind`](value::VcardValueKind)) whose wire spelling is reached
//! through `FromStr` and `Deref`. A property is a
//! [`VcardProp`](prop::VcardProp) struct of a name, parameters and one value;
//! its parameters and value are open payload enums
//! ([`VcardParam`](param::VcardParam), [`VcardValue`](value::VcardValue)) with
//! an `Unknown` variant, so anything outside the model survives.
//!
//! The syntax tree ([`tree`], gated behind the `parser` feature, on by default)
//! is everything byte-faithful. Its hub is [`VcardCst`](tree::cst::VcardCst), a
//! tree of generic nodes that reproduces the wire bytes exactly. Parsing fills
//! a CST; [`decode`](tree::codec::decode) projects it onto the decoded
//! [`Vcard`](vcard::Vcard); [`encode`](tree::codec::encode) (and `From<Vcard>`)
//! projects the model back to a canonical CST. Per-property lens markers
//! ([`VcardPropLens`](tree::prop::VcardPropLens)) read or edit a single line
//! through the byte-preserving [`cursor`](tree::value::VcardValueCursor)s, so
//! editing one property leaves every other byte intact.
//!
//! ## The spec layer
//!
//! Each property carries a [`VcardPropSpec`](tree::prop::VcardPropSpec) on its
//! lens marker: the versions it lives in, its cardinality, the value types and
//! parameters it may take per version, and the value type in force given a
//! declared `VALUE`. A single vtable dispatch bridges the open
//! [`VcardPropKind`](prop::VcardPropKind) back to those static specs, so the
//! decoder consults it to pick a value kind,
//! [`validate`](tree::vcard::validate) consults it to check conformance, and
//! the builder consults it to reject illegal construction. Validity and
//! lossiness are orthogonal: a conformant card may still carry extensions, so
//! validity is that runtime predicate, not
//! a second strict type. A card that passes earns a
//! [`Valid`](tree::vcard::validate::Valid) proof, and both `Vcard` and
//! `Valid<Vcard>` convert back into a [`VcardCst`](tree::cst::VcardCst).

extern crate alloc;

pub mod param;
pub mod prop;
pub mod value;
pub mod vcard;
pub mod version;

#[cfg(feature = "parser")]
pub mod tree;
