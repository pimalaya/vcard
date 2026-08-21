# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-08-21

### Fixed

- Fixed the JSContact export tagging every URI-valued resource object with a pre-RFC draft type name.

  `MediaResource`, `CryptoResource`, `CalendarResource`, `LinkResource` and `DirectoryResource` are now `Media`, `CryptoKey`, `Calendar`, `Link` and `Directory`, as RFC 9553 §2.6 registers them; a strict server rejected the draft spelling, so a contact carrying so much as a `URL` could not be written over JMAP. Import still ignores `@type`, so an earlier Card converts back unchanged.

## [0.2.0] - 2026-08-08

### Changed

- Renamed the public items that did not carry the `Vcard` domain prefix: `Codec` to `VcardCodec`, `Escaper` to `VcardEscaper`, `Valid` to `VcardValid`, and `VcardUnknownValue` to `VcardValueUnknown`.

  The property and parameter lens markers keep their wire spelling (`FN`, `ADR`, `SORT_AS`), a deliberate deviation documented in [CONTRIBUTING.md](./CONTRIBUTING.md).

- Renamed the `FromStr` errors onto the `<Domain><Target><Verb><Ext>` pattern: `ParseVcardPropKindError` to `VcardPropKindParseError`, and likewise `ParseVcardParamKindError`, `ParseVcardValueKindError`, `ParseVcardVersionError`, `ParseJcardError` (now `VcardJcardParseError`) and `ParseJscontactError` (now `VcardJscontactParseError`).

- Moved the flattened re-exports onto their real module paths.

  The lens, spec, cardinality, node and cursor types now carry the module that owns them (`tree::prop::lens::VcardPropLens`, `tree::value::cursor::VcardValueCursor`), and the `#[doc(inline)] pub use` re-exports that hid those modules are gone.

- Bumped `base64` from 0.22 to 0.23, which moves the `base64::DecodeError` that `VcardBinary::decode_base64` returns.

- Pinned every dependency and dev-dependency to `default-features = false` with only the features it needs, so `encoding_rs` is now pulled in its alloc-only mode.

- Replaced the docs/ folder with a cairn/ folder holding the living spec, the in-flight proposals and the dated history, activated by [AGENTS.md](./AGENTS.md). The benchmark methodology and numbers moved to [benches/README.md](./benches/README.md).

## [0.1.0] - 2026-07-16

### Added

- Added the version-agnostic decoded model, available without the `parser` feature.

  A `Vcard` is a version plus a list of `VcardProp` (a name, parameters and one value). `VcardParam` and `VcardValue` are open enums with an `Unknown` arm, beside the structured value types `VcardN`, `VcardAdr`, `VcardGender`, `VcardOrg`, `VcardClientPidMap` and the scalar ones.

- Added the closed `VcardPropKind`, `VcardParamKind`, `VcardValueKind` and `VcardVersion` vocabularies.

  Each reaches its wire spelling through `FromStr` and `Deref<str>`, and `VcardValue::kind` / `VcardParam::kind` recover the kind of an open value or parameter. An unknown name is kept verbatim, and an unrecognised or missing card version normalises to `VcardVersion::V4_0`.

- Added the per-property `VcardPropSpec` contract on the lens markers.

  It declares the versions a property lives in, its `VcardPropCardinality` (version-forked where RFC 6350 forks it, as for `FN` and `N`), the value kinds and parameters it allows per version, and the value kind in force for a declared `VALUE`.

- Added `Vcard::validate`, an RFC 6350 conformance check over the decoded model.

  It verifies per-version property existence, value kind, version-aware parameters and cardinality, including required-but-absent, while extensions pass. A card that passes earns `Valid<Vcard>`, which only validation can mint.

- Added `VcardPropBuilder`, a version-aware, spec-driven builder for strict construction.

  It pins the property name and reuses the per-property validation, rejecting a disallowed value kind or known parameter while still accepting extension parameters.

- Added the byte-faithful syntax tree behind the `parser` feature (on by default).

  `VcardCst` parses bytes or text into a tree that reproduces the wire exactly, decodes onto the model, encodes back, and edits one property in place through per-property lenses and byte-preserving cursors. `to_bytes` is the faithful serializer; `Display` is a convenience that is lossy only for a non-UTF-8 value.

- Added raw-byte value handling for foreign character sets.

  A property value is kept as bytes, so a vCard 2.1 `CHARSET` survives byte for byte, while a name or parameter must be UTF-8; `VcardValueCursor::bytes` and `set_bytes` are the escape hatch.

- Added multi-card and bare-record parsing.

  `VcardCst::parse_many` iterates every card in a file, and `VcardCst::parse` also accepts a bare RFC 2425 directory record with no `BEGIN`/`END` envelope.

- Added `VcardCst::agent`, which re-parses the vCard embedded in an `AGENT` property, opting into exactly one level of nesting and never recursively.

- Added the three-way merge `tree::merge::merge`.

  It diffs two divergent edits against their common base into per-side `VcardMergeAction` lists, matching property instances by PID then equality then position, and replays the right side onto a clone of the left byte-preservingly. A divergent same-field change becomes a `VcardMergeConflict`, where the left action wins except an update over a removal.

- Added opt-in content-decoding features, each backed by a `no_std` crate.

  `quoted-printable` decodes `=XX` octets, `base64` decodes inline binary values, and `encoding` transcodes a foreign `CHARSET`. The core keeps such values raw and their parameters intact, so nothing is silently transcoded.

- Added the RFC 7095 jCard codec behind the `jcard` feature (off by default, requires `parser`).

  `Vcard::to_jcard` and `from_jcard` project the decoded model to and from a `serde_json::Value`, resolving value kinds through the property specs. Export follows the RFC; import accepts anything structurally sound.

- Added the RFC 9554 vocabulary, modeled first-class.

  The `CREATED`, `GRAMGENDER`, `LANGUAGE`, `PRONOUNS` and `SOCIALPROFILE` properties and the nine new parameters (plus the RFC 9555 `JSPROP` and `JSPTR`) each gain a lens marker and spec, and the property-agnostic ones are allowed on any 4.0 property. `VcardAdr` carries all eighteen address components, writing the extended slots only when one is filled.

- Added the RFC 9555 JSContact conversion behind the `jscontact` feature (off by default, requires `jcard`).

  `Vcard::to_jscontact` and `from_jscontact` convert to and from an RFC 9553 Card, infallibly aside from a non-object import root. Unmappable properties are preserved in `vCardProps`, leftover parameters in `vCardParams`, and unknown Card members as `JSPROP` properties.

[unreleased]: https://github.com/pimalaya/vcard/compare/v0.2.1..HEAD
[0.2.1]: https://github.com/pimalaya/vcard/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/vcard/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/vcard/compare/root..v0.1.0
