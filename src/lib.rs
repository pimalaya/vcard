#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # vcard-rs
//!
//! One version-agnostic vCard library: a decoded model and a byte-faithful
//! syntax tree, reading and writing vCard 2.1 (versitcard), 3.0 (RFC 2426) and
//! 4.0 (RFC 6350) alike. The version is a decoded indicator, never a type
//! parameter: the tree ignores it, and only the codec and the per-property spec
//! branch on it. The crate is `no_std` (with `alloc`), its core is
//! dependency-free, and every dependency sits behind a
//! [cargo feature](#cargo-features).
//!
//! This header is the architecture; the behaviour behind it is specified
//! capability by capability in the repository's cairn/spec folder.
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
//! Parsing is maximally liberal: any real card round-trips byte for byte, its
//! folds, its blank lines and its QUOTED-PRINTABLE soft breaks included, and an
//! `Unknown` arm on every open enum carries vocabulary no version defines into
//! the model.
//!
//! Strictness lives on the way out: the builder refuses to construct a property
//! the spec forbids, and [`validate`](tree::vcard::validate) checks a decoded
//! card against its version's RFC contract.
//! ## The two layers
//!
//! The decoded model ([`vcard`], [`version`], [`prop`], [`param`], [`value`])
//! is pure data, with no dependency on the syntax side, so it can be depended
//! on alone. Property names, parameter names and value kinds are closed enums
//! ([`VcardPropKind`](prop::VcardPropKind),
//! [`VcardParamKind`](param::VcardParamKind),
//! [`VcardValueKind`](value::VcardValueKind)) reaching their wire spelling
//! through `FromStr` and `Deref`; a [`VcardProp`](prop::VcardProp) is a name,
//! its parameters and one value, the last two open payload enums
//! ([`VcardParam`](param::VcardParam), [`VcardValue`](value::VcardValue)) with
//! an `Unknown` arm.
//!
//! The syntax tree ([`tree`], behind the default `parser` feature) is
//! everything byte-faithful. Its hub is [`VcardCst`](tree::cst::VcardCst),
//! generic nodes reproducing the wire exactly:
//! [`parse`](tree::cst::VcardCst::parse) reads one card (or a bare RFC 2425
//! record), [`parse_many`](tree::cst::VcardCst::parse_many) iterates a file,
//! and [`decode`](tree::codec::decode) / [`encode`](tree::codec::encode)
//! project between tree and model.
//!
//! A line is logical, its folding resolved for every layer above and recorded
//! on its [`wire`](tree::wire) shape, so output puts it back where it was and
//! an edit that moves the bytes drops it.
//!
//! Per-property lens markers
//! ([`VcardPropLens`](tree::prop::lens::VcardPropLens)) edit one line through
//! byte-preserving [`cursor`](tree::value::cursor::VcardValueCursor)s, and the
//! three-way [`VcardMerge`](tree::merge::VcardMerge) reconciles two divergent
//! copies on those same edits.
//!
//! A property value is raw bytes, so a foreign charset (a vCard 2.1 `CHARSET`)
//! survives; a name or parameter must be UTF-8, as every grammar guarantees.
//! [`to_bytes`](tree::cst::VcardCst::to_bytes) is therefore the faithful
//! serializer, and [`Display`](core::fmt::Display) a convenience that is lossy
//! only for a non-UTF-8 value.
//!
//! ## The spec layer
//!
//! Each property carries a [`VcardPropSpec`](tree::prop::spec::VcardPropSpec)
//! on its lens marker, declaring per version what it allows, and one vtable
//! dispatch bridges the open [`VcardPropKind`](prop::VcardPropKind) back to
//! those static specs. That one source of truth has three readers: the decoder
//! picks a value kind from it, [`validate`](tree::vcard::validate) checks
//! conformance against it, and the builder rejects illegal construction with
//! it. A card that passes earns a
//! [`VcardValid`](tree::vcard::validate::VcardValid) proof; validity is a
//! runtime predicate rather than a second type, since a conformant card may
//! still carry extensions.
//!
//! ## Content encodings
//!
//! The core transforms no content: a `QUOTED-PRINTABLE` or `BASE64` encoding
//! and a `CHARSET` are surfaced raw with their parameters kept, so nothing is
//! silently transcoded. Only the value grammar (escapes and folding) is
//! resolved, and put back from the line's wire shape on output. Decoding is
//! opt-in, one small `no_std` crate per feature:
//! [`quoted_printable`](tree::value::cursor::VcardValueCursor::quoted_printable)
//! and [`charset`](tree::value::cursor::VcardValueCursor::charset) on the value
//! cursor, [`decode_base64`](value::binary::VcardBinary::decode_base64) on the
//! binary value.
//!
//! ## Cargo features
//!
//! - `parser` (default): the byte-faithful [`tree`] and its codec, via the
//!   `memchr` crate. Everything under [`tree`] is gated on it; the decoded
//!   model is always available.
//! - `jcard`: the RFC 7095 jCard codec on the decoded model
//!   ([`to_jcard`](vcard::Vcard::to_jcard) /
//!   [`from_jcard`](vcard::Vcard::from_jcard)), via the `serde_json` crate
//!   (`no_std`, alloc-only). Implies `parser` for the property specs.
//! - `jscontact`: the RFC 9555 conversion between the decoded model and an RFC
//!   9553 JSContact Card ([`to_jscontact`](vcard::Vcard::to_jscontact) /
//!   [`from_jscontact`](vcard::Vcard::from_jscontact)). Implies `jcard`, whose
//!   syntax carries the vCardProps / vCardParams escape hatches, and pulls no
//!   crate of its own.
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
