#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # vcard-rs
//!
//! One version-agnostic vCard library: a decoded model and a byte-faithful
//! syntax tree that read and write vCard 2.1 (versitcard), 3.0 (RFC 2426) and
//! 4.0 (RFC 6350) alike.
//!
//! The version is a decoded indicator, never a type parameter: the tree
//! ignores it, and only the codec and the per-property spec branch on it.
//!
//! The crate is `no_std` (with `alloc`), its core is dependency-free, and
//! every dependency sits behind a [cargo feature](#cargo-features).
//!
//! This header is the architecture; the behaviour behind it is specified
//! capability by capability in the repository's cairn/spec folder.
//!
//! ## Postel's law
//!
//! Parsing is maximally liberal: any real card round-trips byte for byte, its
//! folds, its blank lines and its `QUOTED-PRINTABLE` soft breaks included, and
//! an `Unknown` arm on every open enum carries vocabulary no version defines
//! into the model.
//!
//! Strictness lives on the way out: the builder refuses to construct a
//! property the spec forbids, and [`validate`](tree::vcard::validate) checks a
//! decoded card against its version's RFC contract.
//!
//! ## The two layers
//!
//! The decoded model ([`vcard`], [`version`], [`prop`], [`param`], [`value`])
//! is pure data with no dependency on the syntax side, so it can be depended
//! on alone.
//!
//! Property and parameter names and value kinds are closed identity enums
//! ([`VcardPropKind`], [`VcardParamKind`], [`VcardValueKind`]) whose wire
//! spelling is reached through `FromStr` and `Deref`.
//!
//! A card is a [`Vcard`]: a version indicator and its properties, in source
//! order.
//!
//! A property is a [`VcardProp`] of a name, its parameters and one value, the
//! last two open payload enums ([`VcardParam`], [`VcardValue`]) with an
//! `Unknown` arm, so anything outside the model survives.
//!
//! The syntax tree ([`tree`], behind the default `parser` feature) is
//! everything byte-faithful. Its hub is [`VcardCst`], generic nodes
//! reproducing the wire bytes exactly.
//!
//! A line is logical, its folding resolved for every layer above and recorded
//! on its [`VcardWire`] shape, so serialization puts it back where it was and
//! an edit that moves the bytes drops it.
//!
//! [`parse`] reads one card, or a bare RFC 2425 record carrying no `BEGIN` and
//! `END` envelope, and [`parse_many`] iterates a multi-card file.
//!
//! [`decode`] projects a CST onto the decoded [`Vcard`], and [`encode`]
//! projects the model back to a canonical CST.
//!
//! Per-property lens markers ([`VcardPropLens`]) read or edit one line through
//! the byte-preserving [`cursor`]s, and the three-way [`VcardMerge`]
//! reconciles two divergent copies on those same edits.
//!
//! A property value is raw bytes, so a foreign charset (a vCard 2.1 `CHARSET`)
//! survives; a name or a parameter must be UTF-8, as every grammar guarantees.
//! [`to_bytes`] is therefore the faithful serializer, and `Display` a
//! convenience that is lossy only for a non-UTF-8 value.
//!
//! ## The spec layer
//!
//! Each property carries a [`VcardPropSpec`] on its lens marker, declaring per
//! version the value kinds and parameters it allows, and one vtable dispatch
//! bridges the open [`VcardPropKind`] back to those static specs.
//!
//! That one source of truth has three readers: the decoder picks a value kind
//! from it, [`validate`](tree::vcard::validate) checks conformance against it,
//! and the builder rejects illegal construction with it.
//!
//! A card that passes earns a [`VcardValid`] proof. Validity is a runtime
//! predicate rather than a second type, since a conformant card may still
//! carry extensions.
//!
//! ## Content encodings
//!
//! The core transforms no content: a `QUOTED-PRINTABLE` or `BASE64` encoding
//! and a `CHARSET` are surfaced raw with their parameters kept, so nothing is
//! silently transcoded.
//!
//! Only the value grammar (escapes and folding) is resolved, and put back from
//! the line's wire shape on output.
//!
//! Decoding is opt-in, one small `no_std` crate per feature: the value
//! [`cursor`] exposes `quoted_printable` and `charset`, and the binary value
//! exposes [`decode_base64`].
//!
//! ## The JSON representations
//!
//! [`jcard`] is the RFC 7095 spelling of this model in JSON, member for
//! member.
//!
//! [`jscontact`] is the RFC 9553 data model, which is a different model: a
//! card is a Card object of named members, an `ADR` line is a structured
//! Address, and a `TYPE` parameter is a context.
//!
//! Both take a raw [`serde_json::Value`] at the boundary rather than a serde
//! implementation, since one model with two JSON spellings is exactly what
//! serde cannot key. jCard normalizes rather than preserves, while JSContact
//! is lossless through the RFC 9555 escape hatches.
//!
//! ## Cargo features
//!
//! `parser` (default) brings the byte-faithful [`tree`] and its codec, via the
//! `memchr` crate. Everything under [`tree`] is gated on it; the decoded model
//! is always available.
//!
//! Three content decoders are default too, one small crate each:
//! `quoted-printable` decodes `QUOTED-PRINTABLE` value octets, `base64`
//! decodes inline `BASE64` binary values, and `encoding` transcodes a foreign
//! `CHARSET` to text through `encoding_rs` (the WHATWG Encoding Standard).
//!
//! `jcard` adds the RFC 7095 JSON representation, via the `serde_json` crate,
//! and implies `parser` for the property specs. `jscontact` adds the RFC 9555
//! conversion to the RFC 9553 Card, implies `jcard`, whose syntax carries the
//! escape hatch, and pulls no crate of its own.
//!
//! [`VcardPropKind`]: prop::VcardPropKind
//! [`VcardParamKind`]: param::VcardParamKind
//! [`VcardValueKind`]: value::VcardValueKind
//! [`Vcard`]: vcard::Vcard
//! [`VcardProp`]: prop::VcardProp
//! [`VcardParam`]: param::VcardParam
//! [`VcardValue`]: value::VcardValue
//! [`VcardCst`]: tree::cst::VcardCst
//! [`VcardWire`]: tree::wire::VcardWire
//! [`parse`]: tree::cst::VcardCst::parse
//! [`parse_many`]: tree::cst::VcardCst::parse_many
//! [`decode`]: tree::codec::decode
//! [`encode`]: tree::codec::encode
//! [`VcardPropLens`]: tree::prop::lens::VcardPropLens
//! [`cursor`]: tree::value::cursor::VcardValueCursor
//! [`VcardMerge`]: tree::merge::VcardMerge
//! [`to_bytes`]: tree::cst::VcardCst::to_bytes
//! [`VcardPropSpec`]: tree::prop::spec::VcardPropSpec
//! [`VcardValid`]: tree::vcard::validate::VcardValid
//! [`decode_base64`]: value::binary::VcardBinary::decode_base64

extern crate alloc;

#[cfg(feature = "jcard")]
#[cfg_attr(docsrs, doc(cfg(feature = "jcard")))]
pub mod jcard;
#[cfg(feature = "jscontact")]
#[cfg_attr(docsrs, doc(cfg(feature = "jscontact")))]
pub mod jscontact;
pub mod param;
pub mod prop;
pub mod value;
pub mod vcard;
pub mod version;

#[cfg(feature = "parser")]
pub mod tree;
