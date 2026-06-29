//! # vCard 2.1 (versitcard)
//!
//! A vCard 2.1 implementation built around **two representations of a card,
//! bridged by lenses** (a copy of [`crate::v40`], being adapted to the 2.1
//! grammar and semantics). The code is the living architecture; this is the map.
//!
//! - The **syntax tree** ([`tree`]) — [`VcardCst`](tree::cst::VcardCst), a
//!   generic, byte-faithful concrete syntax tree of lines, parameters,
//!   components and leaves. It knows *no* property semantics. It parses from
//!   raw text, serializes back exactly, and supports byte-preserving in-place
//!   edits.
//! - The **decoded model** — pure data types ([`Vcard`](vcard::Vcard),
//!   [`VcardProp`](prop::VcardProp), [`VcardValue`](value::VcardValue), ...),
//!   one concern per sibling module, always unescaped, with an `Unknown` arm so
//!   unknown properties and parameters round-trip. You construct these directly
//!   (every field is `pub`); there is no builder type.
//!
//! ## The four bridges
//!
//! ```text
//!            parse                          decode
//!   &str  ─────────▶   VcardCst   ──────────────────▶   Vcard
//!         ◀─────────              ◀──────────────────
//!          to_string                     encode
//! ```
//!
//! - **parse** — [`VcardCst::parse`](tree::cst::VcardCst::parse) borrows the
//!   source; leaves point into it, so an unfolded card round-trips byte for
//!   byte. Line folding is normalised away: a folded line is unfolded into an
//!   owned line and serialized unfolded.
//! - **to_string** — [`Display`](core::fmt::Display) on `VcardCst` emits the
//!   bytes verbatim (parsed) or canonically (built).
//! - **decode** — [`VcardCst::decode`](tree::cst::VcardCst::decode) projects
//!   the whole card into the typed [`Vcard`](vcard::Vcard).
//! - **encode** — [`Vcard::encode`](vcard::Vcard::encode) (and
//!   [`Display`](core::fmt::Display) for `Vcard`) renders a decoded card back
//!   to syntax, canonically.
//!
//! ## Typed access and editing
//!
//! A property is reached by type: [`VcardCst::prop`](tree::cst::VcardCst::prop)
//! (`card.prop::<N>()`) decodes one property;
//! [`prop_mut`](tree::cst::VcardCst::prop_mut) returns a typed cursor that
//! edits *in place*, escaping on write and leaving every untouched leaf (and
//! parameter) byte for byte intact.  [`push`](tree::cst::VcardCst::push) and
//! [`remove`](tree::cst::VcardCst::remove) add and drop properties; on a parsed
//! card they preserve the existing bytes, so "build" is just "edit an empty
//! card".
//!
//! ## The lens, and where semantics live
//!
//! The syntax tree is deliberately dumb. *All* per-property and per-parameter
//! knowledge lives in the lens markers
//! ([`VcardPropLens`](tree::prop::VcardPropLens) /
//! [`VcardParamLens`](tree::param::VcardParamLens)): a marker
//! ([`tree::prop::n::N`], [`tree::param::encoding::ENCODING`]) is a zero-sized type
//! pinning a wire name to the decoded value it produces, plus the `decode`/`encode`
//! projections. Each property has its own hand-written module under
//! [`tree::prop`]; most share a value kind
//! ([`VcardText`](value::text::VcardText), [`VcardUri`](value::uri::VcardUri),
//! ...) and the generic [`VcardValueCursor`](tree::cursor::VcardValueCursor),
//! while the structured ones ([`N`](value::n::VcardN),
//! [`ADR`](value::adr::VcardAdr), ...) carry a cursor that names their
//! components. The wire names have a single source of truth: the `VCARD_*` consts in
//! [`prop`] and [`param`].
//!
//! ## Invariants
//!
//! - The syntax tree never interprets a value; only lenses do.
//! - Escaping and unescaping happen *only* at the codec boundary
//!   ([`tree::encode`] / [`tree::decode`]).
//! - Wire names have a single source of truth: the `VCARD_*` consts in [`prop`]
//!   and [`param`].
//! - Editing a parsed card is byte-preserving; building and encoding are
//!   canonical.
//!
//! ## Module map
//!
//! - the decoded model (no syntax dependency): [`vcard`] (the card),
//!   [`version`], [`prop`] (property + names), [`param`] (parameter + names),
//!   [`value`] (the value kinds, one per submodule).
//! - [`tree`] — everything syntactic, gated behind the `parser` feature:
//!   [`cst`](tree::cst) (the tree, parse, serialize, typed access),
//!   [`leaf`](tree::leaf) / [`line`](tree::line) / [`value`](tree::value) /
//!   [`param`](tree::param) (the nodes), [`prop`](tree::prop) /
//!   [`param`](tree::param) (per-name lens markers and their `VcardPropLens` /
//!   `VcardParamLens` contracts), [`cursor`](tree::cursor) (in-place editing),
//!   [`decode`](tree::decode) / [`encode`](tree::encode) (the model bridge and
//!   value codec), and [`error`](tree::error) (the parse errors).
//!
//! ## Status
//!
//! Every vCard 2.1 property and parameter is modelled. Simple values share a
//! small set of value kinds; the structured ones (`N`, `ADR`, `ORG`, `GEO`) and
//! the binary ones (`PHOTO`, `LOGO`, `SOUND`, `KEY`, via
//! [`VcardBinary`](value::binary::VcardBinary)) get bespoke types; anything
//! unrecognised round-trips through the `Unknown` arms. [`tree`] is gated behind
//! the `parser` feature, so the decoded model can be depended on without the
//! syntax layer.
//!
//! ## Limitations
//!
//! Input is parsed as UTF-8. The `CHARSET` parameter is preserved on the syntax
//! side but not used to transcode, so a non-UTF-8 2.1 card (e.g. latin-1 plus
//! `QUOTED-PRINTABLE`) is not yet fully supported.

pub mod param;
pub mod prop;
pub mod value;
pub mod vcard;
pub mod version;

#[cfg(feature = "parser")]
pub mod tree;
