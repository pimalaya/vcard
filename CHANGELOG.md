# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fixed the JSContact export tagging every URI-valued resource object with a pre-RFC draft type name.

  `MediaResource` (PHOTO, LOGO, SOUND), `CryptoResource` (KEY), `CalendarResource` (CALURI, FBURL), `LinkResource` (URL) and `DirectoryResource` (SOURCE) are now `Media`, `CryptoKey`, `Calendar`, `Link` and `Directory`, as RFC 9553 §2.6 registers them. A strict server rejects the draft spelling outright, so a contact carrying so much as a `URL` could not be written over JMAP. Import is unchanged and still ignores `@type`, so a Card produced by an earlier version converts back exactly as before.

## [0.2.0] - 2026-08-08

### Changed

- Renamed the public items that did not carry the `Vcard` domain prefix: `Codec` to `VcardCodec`, `Escaper` to `VcardEscaper`, `Valid` to `VcardValid`, and `VcardUnknownValue` to `VcardValueUnknown`.

  The property and parameter lens markers keep their wire spelling (`FN`, `ADR`, `SORT_AS`), a deliberate deviation documented in [CONTRIBUTING.md](./CONTRIBUTING.md).

- Renamed the `FromStr` errors onto the `<Domain><Target><Verb><Ext>` pattern: `ParseVcardPropKindError` to `VcardPropKindParseError`, and likewise `ParseVcardParamKindError`, `ParseVcardValueKindError`, `ParseVcardVersionError`, `ParseJcardError` (now `VcardJcardParseError`) and `ParseJscontactError` (now `VcardJscontactParseError`).

- Moved the flattened re-exports onto their real module paths.

  `tree::param::lens`, `tree::param::node`, `tree::prop::cardinality`, `tree::prop::lens`, `tree::prop::spec`, `tree::value::cursor` and `tree::value::node` are public modules, and the module name is now part of the public path (`tree::prop::lens::VcardPropLens`, `tree::value::cursor::VcardValueCursor`). The `#[doc(inline)] pub use` re-exports that hid them are gone.

- Bumped `base64` from 0.22 to 0.23, which moves the `base64::DecodeError` that `VcardBinary::decode_base64` returns.

- Pinned every dependency and dev-dependency to `default-features = false` with only the features it needs, so `encoding_rs` is now pulled in its alloc-only mode.

- Replaced the docs/ folder with a cairn/ folder holding the living spec, the in-flight proposals and the dated history, activated by [AGENTS.md](./AGENTS.md). The benchmark methodology and numbers moved to [benches/README.md](./benches/README.md).

## [0.1.0] - 2026-07-16

### Added

- Added the version-agnostic decoded model, available without the `parser` feature.

  A `Vcard` is a version plus a list of `VcardProp` (a name, parameters and one value). Parameters and values are the open `VcardParam` and `VcardValue` enums, each with an `Unknown` arm so anything outside the model survives, alongside the structured value types `VcardN`, `VcardAdr`, `VcardGender`, `VcardOrg`, `VcardGeo`, `VcardClientPidMap`, `VcardBinary`, `VcardDateAndOrTime`, `VcardTimestamp`, `VcardUtcOffset`, `VcardText`, `VcardTextList`, `VcardUri` and `VcardLanguageTag`.

- Added the closed `VcardPropKind`, `VcardParamKind`, `VcardValueKind` and `VcardVersion` vocabularies.

  Each reaches its wire spelling through `FromStr` and `Deref<str>`, and `VcardValue::kind` / `VcardParam::kind` recover the kind of an open value or parameter. `VcardPropName` holds either a known `VcardPropKind` or a verbatim unknown name; an unrecognised or missing card version normalises to `VcardVersion::V4_0` in the decoded model, while byte-faithful round-tripping stays on the syntax tree.

- Added the per-property `VcardPropSpec` contract on the lens markers.

  It declares the versions a property lives in, its `VcardPropCardinality` (version-forked where RFC 6350 forks it, as for `FN` and `N`), the value kinds and parameters it allows per version, and the value kind in force for a declared `VALUE`. `VcardPropKind::ALL` enumerates every known property.

- Added `Vcard::validate`, an RFC 6350 conformance check over the decoded model.

  It verifies per-version property existence, value kind, version-aware parameters and cardinality (including required-but-absent) while still permitting extensions. A card that passes earns `Valid<Vcard>`, a proof only validation can mint (`TryFrom<Vcard>`); both `Vcard` and `Valid<Vcard>` convert into a `VcardCst`.

- Added `VcardPropBuilder`, a version-aware, spec-driven builder for strict construction.

  It pins the property name and reuses the per-property validation, rejecting a disallowed value kind or known parameter through `Result` while still accepting extension parameters.

- Added the byte-faithful syntax tree behind the `parser` feature (on by default).

  `VcardCst` parses bytes or text into a tree that reproduces the wire exactly, decodes onto the model, encodes back to a canonical tree, and edits one property in place through per-property lenses and byte-preserving cursors. `to_bytes` is the byte-faithful serializer, while `Display` / `to_string` is a convenience that is lossy only for a non-UTF-8 value.

- Added raw-byte value handling for foreign character sets.

  A property value is kept as bytes, so a value in a vCard 2.1 `CHARSET` survives byte for byte, while a name or parameter must be UTF-8 (a non-UTF-8 one is a parse error). `VcardValueCursor::bytes` and `set_bytes` are the byte escape hatch.

- Added multi-card and bare-record parsing.

  `VcardCst::parse_many` iterates every card in a file, and `VcardCst::parse` also accepts a bare RFC 2425 directory record with no `BEGIN`/`END` envelope.

- Added `VcardCst::agent`, which re-parses the vCard embedded in an `AGENT` property, opting into exactly one level of nesting and never recursively.

- Added the three-way merge `tree::merge::merge`.

  It diffs two divergent edits of a card against their common base into per-side `VcardMergeAction` lists, matches property instances by PID then equality then position, and replays the right side onto a clone of the left through the byte-preserving edit layer. Divergent same-field changes are surfaced as `VcardMergeConflict`, where the left action wins except an update over a removal.

- Added opt-in content-decoding features, each backed by a `no_std` crate.

  `quoted-printable` decodes `=XX` octets (`VcardValueCursor::quoted_printable`), `base64` decodes inline binary values (`VcardBinary::decode_base64`), and `encoding` transcodes a foreign `CHARSET` (`VcardValueCursor::charset`, via `encoding_rs`). The core keeps such values raw and their parameters intact, so nothing is silently lost or transcoded.

- Added the RFC 7095 jCard codec behind the `jcard` feature (off by default, requires `parser`).

  `Vcard::to_jcard` writes the decoded model as a `serde_json::Value` and `Vcard::from_jcard` reads one back, borrowing the JSON and resolving value kinds through the property specs. Export follows the RFC (lowercased names, group prefix to the `group` parameter, `VALUE` to the type slot, dates re-spelled extended); import accepts anything structurally sound. `serde_json` is pulled in its `no_std` alloc-only mode.

- Added the RFC 9554 vocabulary, modeled first-class.

  The `CREATED`, `GRAMGENDER`, `LANGUAGE`, `PRONOUNS` and `SOCIALPROFILE` properties and the `AUTHOR`, `AUTHOR-NAME`, `CREATED`, `DERIVED`, `PHONETIC`, `PROP-ID`, `SCRIPT`, `SERVICE-TYPE` and `USERNAME` parameters (plus the RFC 9555 `JSPROP` property and `JSPTR` parameter) each gain a lens marker and spec, and validation allows the property-agnostic RFC 9554 parameters on any 4.0 property. `VcardAdr` carries the full eighteen address components, writing the eleven extended slots only when one is filled.

- Added the RFC 9555 JSContact conversion behind the `jscontact` feature (off by default, requires `jcard`).

  `Vcard::to_jscontact` converts the decoded model into an RFC 9553 Card `serde_json::Value` and `Vcard::from_jscontact` converts one back; both directions are infallible aside from a non-object import root. `TYPE`, `PREF` and `PROP-ID` map to contexts and features, `pref`, and the object key, while unmappable properties are preserved in `vCardProps`, leftover parameters in `vCardParams` (both in jCard syntax) and unknown Card members as `JSPROP` properties.

[unreleased]: https://github.com/pimalaya/vcard/compare/v0.2.0..HEAD
[0.2.0]: https://github.com/pimalaya/vcard/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/vcard/compare/root..v0.1.0
