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
//! `alloc`); its core is dependency-free, and the only dependencies are the
//! small `no_std` crates behind the opt-in content-decoding features
//! ([Cargo features](#cargo-features)).
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
//! tree of generic nodes that reproduces the wire bytes exactly.
//! [`parse`](tree::cst::VcardCst::parse) accepts bytes or a string and reads
//! one card (or a bare RFC 2425 record with no `BEGIN`/`END`);
//! [`parse_many`](tree::cst::VcardCst::parse_many) iterates a multi-card file.
//! [`decode`](tree::codec::decode) projects a CST onto the decoded
//! [`Vcard`](vcard::Vcard); [`encode`](tree::codec::encode) (and `From<Vcard>`)
//! projects the model back to a canonical CST. Per-property lens markers
//! ([`VcardPropLens`](tree::prop::lens::VcardPropLens)) read or edit a single
//! line through the byte-preserving
//! [`cursor`](tree::value::cursor::VcardValueCursor)s, so editing one property
//! leaves every other byte intact. The three-way [`merge`](tree::merge::merge)
//! builds on those same edits to reconcile two divergent copies of a card
//! against their common base, reporting each side's changes and their
//! conflicts.
//!
//! A property *value* is held as raw bytes, so a value in a foreign charset (a
//! vCard 2.1 `CHARSET`) survives byte for byte; a name or parameter must be
//! UTF-8, as every version's grammar guarantees. Because of that,
//! [`to_bytes`](tree::cst::VcardCst::to_bytes) is the byte-faithful serializer,
//! while [`Display`](core::fmt::Display) / `to_string` are a convenience that
//! is lossy only for a non-UTF-8 value.
//!
//! ## The spec layer
//!
//! Each property carries a [`VcardPropSpec`](tree::prop::spec::VcardPropSpec)
//! on its lens marker: the versions it lives in, its cardinality, the value
//! types and parameters it may take per version, and the value type in force
//! given a declared `VALUE`. A single vtable dispatch bridges the open
//! [`VcardPropKind`](prop::VcardPropKind) back to those static specs, so the
//! decoder consults it to pick a value kind,
//! [`validate`](tree::vcard::validate) consults it to check conformance, and
//! the builder consults it to reject illegal construction. Validity and
//! lossiness are orthogonal: a conformant card may still carry extensions, so
//! validity is that runtime predicate, not a second strict type. A card that
//! passes earns a [`VcardValid`](tree::vcard::validate::VcardValid) proof, and
//! both `Vcard` and `VcardValid<Vcard>` convert back into a
//! [`VcardCst`](tree::cst::VcardCst).
//!
//! ## Content encodings
//!
//! The core transforms no content: a `QUOTED-PRINTABLE` or `BASE64` transfer
//! encoding and a `CHARSET` are surfaced raw, with their parameters kept, so
//! nothing is silently lost or transcoded (only the value grammar, escapes and
//! line folding, is resolved). Decoding them is opt-in, one small `no_std`
//! crate per feature (see [Cargo features](#cargo-features)):
//! [`quoted_printable`](tree::value::cursor::VcardValueCursor::quoted_printable)
//! and [`charset`](tree::value::cursor::VcardValueCursor::charset) on the value
//! cursor, and [`decode_base64`](value::binary::VcardBinary::decode_base64) on
//! the binary value.
//!
//! ## Cargo features
//!
//! - `parser` (default): the byte-faithful [`tree`] and its codec. Everything
//!   under [`tree`] is gated on it; the decoded model is always available.
//! - `jcard` (opt-in): the RFC 7095 jCard codec on the decoded model
//!   ([`to_jcard`](vcard::Vcard::to_jcard) /
//!   [`from_jcard`](vcard::Vcard::from_jcard)), via the `serde_json` crate
//!   (`no_std`, alloc-only). Requires `parser` for the property specs.
//! - `jscontact` (opt-in): the RFC 9555 conversion between the decoded model
//!   and an RFC 9553 JSContact Card
//!   ([`to_jscontact`](vcard::Vcard::to_jscontact) /
//!   [`from_jscontact`](vcard::Vcard::from_jscontact)). Requires `jcard`,
//!   whose syntax carries the vCardProps / vCardParams escape hatches, and
//!   pulls no crate of its own on top of it.
//! - `quoted-printable` (default): decode `QUOTED-PRINTABLE` value octets, via
//!   the `quoted_printable` crate.
//! - `base64` (default): decode inline `BASE64` binary values, via the `base64`
//!   crate.
//! - `encoding` (default): transcode a foreign `CHARSET` value to text, via the
//!   `encoding_rs` crate (the WHATWG Encoding Standard).

extern crate alloc;

#[cfg(feature = "jcard")]
pub mod jcard;
#[cfg(feature = "jscontact")]
pub mod jscontact;
pub mod param;
pub mod prop;
pub mod value;
pub mod vcard;
pub mod version;

#[cfg(feature = "parser")]
pub mod tree;
