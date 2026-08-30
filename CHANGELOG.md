# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed

- Removed the collision preference: the `prefer` field on `VcardMerge` and the `VcardMergeSide` enum. The left side is git's `ours` and always wins; the right side is `theirs`.

  The split was argued as two separately answered questions, the baseline being about bytes and the winner about policy, but no caller ever answered them differently: tCard and neverest both passed `Left`, and nothing in the ecosystem passed `Right`.

  Git already names the arrangement and everybody reads it the same way, so it is the convention rather than a switch with one setting.

  Hard-coding it also retires a mechanism that had quietly become unreachable: a parameter or an addition could only replace the one it beat while the right side was able to win, so the merged card now simply keeps the left side's and reports the collision.

  An update still beats a removal whichever side it came from, which was never the caller's to invert, and a caller wanting the other value still has both actions in the report.

### Added

- Added the `identity` field on `VcardPropPath`, naming which member of a group of same-named properties an action addresses.

- Added `tree::merge::VcardMerge`, a struct carrying the three cards, with a `merge` method replacing the free `merge` function.

  The free function stays as a deprecated shim over it, keeping the left preference, so an existing caller keeps building. It is due for removal once its callers, tCard and neverest among them, build the struct instead.

- Added `tree::merge::VcardMergeSide` and the `prefer` field on `VcardMerge`.

  It says which side's value the merged card carries where both sides changed one field to different things, apart from `left`, which now answers only whose untouched bytes survive. `Left` is the default and the behaviour every merge had before.

  The preference decides that case and no other: an update still beats a removal whichever side it came from, a field one side alone touched is still taken from that side, and the report names the same actions and the same fields whichever way it falls.

- Added `tree::wire::VcardWire`, the wire layout of a content line, and a `wire` field on `VcardLine` carrying it.

  It records what the tokeniser resolves against the line's logical bytes: its folds, the blank lines before it, its QUOTED-PRINTABLE soft breaks, and a dangling `=`. An edit that changes a line's length drops the layout, so the line goes out unfolded rather than folded in the wrong places.

- Added `VcardCst::trailing`, the blank lines a file ends on, so concatenating what `parse_many` yields reproduces the file byte for byte.

### Changed

- Changed the parameter codec to carry an escaping mode, which the parameter side never had.

  `VcardParamNode` gains an `escaper` field, stamped by the parser once `VERSION` is known exactly as a value node's already was, and read by every parameter decode.

  `VcardParam::encode` and `VcardParamLens::encode` take the target `VcardEscaper`, mirroring `VcardProp::encode` and the `VcardCodec` trait, so a parameter is written in the same version's rules it was read in.

- Changed `VcardEscaper::Modern` into `V3_0` and `V4_0`, the two versions it stood for.

  They escape a value identically, so nothing about value escaping moves, but only 4.0 carries the RFC 6868 parameter value encoding and one variant could not say which of the two it was. `has_param_encoding` reports that, and `V4_0` is the default `Modern` was.

- Changed the value node accessors so a truncating read has to name the component it truncates at, replacing `decode_at`, `decode_scalar_at`, `decode_joined_at`, `decode_joined`, `decode_bytes_at`, `set_at` and `set_bytes_at`.

  Reading component zero looks like reading the value and is not: it stops at the first unescaped `;`. Almost every call site passed `0`, and that one shape produced four separate defects in two days across three crates.

  `decode`, `decode_list` and `decode_bytes` now read the whole value; `decode_component` and `decode_component_list` read one `;`-component and always spell out which. `decode_scalar_at` and `decode_bytes_at` are gone, having cut twice, at a `;` and then at a `,`, which no caller wanted.

  `set` and `set_bytes` replace the whole value, `set_component` and `set_component_bytes` name their slot, so a read and the write that follows it address the same thing.

  The generic `VcardValueCursor` follows the same split: `text`, `bytes`, `list` and their setters address the whole value, `component` and `set_component` one slot.

- Changed the three-way merge to match a property instance by its own identity where vCard gives it one, between the `PID` rung and the equality rung.

  A property that may occur more than once and whose value names a thing outside the card is now addressed by that whole value: `EMAIL` by its address, `TEL` by its number, `IMPP`, `URL`, `SOURCE`, `FBURL`, `CALURI`, `CALADRURI`, `PHOTO`, `LOGO`, `SOUND`, `KEY` and `SOCIALPROFILE` by their URI, `MEMBER` and `RELATED` by the entity theirs names.

  An instance carrying an identity is never matched with one carrying none, so a side that reordered or replaced one no longer writes another instance's edit onto it. Changing such a value is therefore one instance leaving and another arriving rather than a rename; a card carrying `PID` keeps the rename, `PID` sitting above.

  An identity is compared lowercased, so a URI scheme (RFC 3986 section 3.1) and a mail host meet whichever case they were written in. Only the comparison normalises: a line goes back out with the bytes the side that wrote it wrote.

- Changed the parser to put back what it unfolds, so a parsed card now serializes back byte for byte, its folds, its blank lines and its QUOTED-PRINTABLE soft breaks included.

  A card exported by Apple, iOS or Google folds heavily, so an untouched round trip used to rewrite every folded line of it. Every layer above the parser still sees one logical line, and every one of the 146 corpus fixtures now comes back identical.

### Fixed

- Fixed parameter value encoding, which read RFC 6350 section 3.4 text escapes into a parameter that has none, and never wrote RFC 6868 at all.

  Section 3.3 gives a parameter value no backslash escapes, which is the whole reason RFC 6868 exists.

  So a backslash a parameter legitimately carried, a Windows path in an `X-` parameter or a `LABEL`, was eaten on the way in and could not be written back, while a real `^n`, `^^` or `^'` from a conforming producer reached the caller with its encoding showing.

  A parameter is now decoded and encoded by RFC 6868 section 3.1: `^n` is a newline, `^^` a caret, `^'` a double quote, any other caret sequence stays literal as section 3.1 requires, and a backslash is content in both directions.

  RFC 6868 updates RFC 6350 and no earlier specification, so the rules apply to vCard 4.0 alone and a 2.1 or 3.0 caret stays a caret; `VcardEscaper::has_param_encoding` is the switch, and a parameter node now carries its card's `VcardEscaper` the way a value node already did.

- Fixed the merge reading two sides that wrote different bytes as one change, which dropped the difference without a word.

  Agreement was decided on the decoded actions and, for a whole value, on the decoded nodes; a decode is not injective, and `\N` and `\n` both unescape to a line break (RFC 6350 section 3.4), so two sides writing a value each way compared equal, the right side's change was skipped as already made, and no conflict was reported.

  Agreement is now byte equality at the granularity of the change itself. The one exception is `TYPE` and `PID` (sections 5.6 and 7), which the specification gives no order and whose items therefore compare as a set, so writing one set in two orders stays one change rather than becoming a conflict.

  The merged bytes are unchanged either way, since the left side keeps its value; what changes is that the divergence is reported.

- Fixed the merge comparing parameters decoded, which hid an edit the decode cannot see.

  A single-valued parameter decodes its first value alone, so `LABEL=Ada,Lovelace` and `LABEL=Ada,Byron` compared equal, the change was never reported, and the edit was dropped without a word.

  Parameters are now compared on their raw nodes, value by value, exactly as values already were, falling back to raw bytes across two cards of different versions that share no decoding. The replay reads the right card's parameter node by name and ordinal for the same reason, a decoded parameter not being a key.

- Fixed every value read that was silently truncating a value it had no business splitting.

  RFC 6350 section 3.4 has a text value escape a `;` or a `,` it means literally, and section 4.2 gives a URI no escaping at all, so an unescaped separator is content.

  A `VcardText`, a `VcardTextList`, a `VcardBinary`, a `VcardDateAndOrTime`, a `VcardTimestamp`, a `VcardLanguageTag` and a `VcardUtcOffset` now keep everything past their first `;`, and the components of the structured values keep the commas inside them: a `CLIENTPIDMAP` URI, a `GENDER` identity and every `ORG` unit were being cut at the first one.

  The generic cursor's `text`, `bytes` and `list` read the whole value, and their setters replace it, so reading a value and writing it straight back no longer leaves the tail of the old one behind.

- Fixed the merge collapsing two parameters of one name into one field, so a side's edit of the second was dropped and a conflict was reported where the two sides had touched nothing in common.

  A property may write one parameter name more than once, `TEL;TYPE=work;TYPE=voice` being the RFC 2426 section 4 example, and the field an action occupied was keyed on the parameter name alone.

  A side rewriting the first `TYPE` and a side rewriting the second therefore contested one field: the preferred side won it, and the other side's edit never reached the merged card.

  Each occurrence is now a field of its own, addressed by its name and by its position among the property's parameters of that name, so both edits land and neither is reported.

  The five parameter-carrying actions (`ParamAdded`, `ParamRemoved`, `ParamChanged`, `ParamItemAdded`, `ParamItemRemoved`) carry that position in a new `index` field.

- Fixed a whole-value change reporting a payload truncated at the value's first `;`, which could make its old and its new read the same.

  Two values are compared on their raw nodes, which is what makes a change past the first `;`-component visible at all, but the reported action was built from the decoded values, and a non-structured value decodes its first `;`-component alone.

  A `NOTE:a;b` changed to `NOTE:a;CHANGED` came back as `ValueChanged { old: Text("a"), new: Text("a") }`, leaving a caller resolving the report unable to see either value, while the merged bytes were right throughout.

  A value whose decoding does not say what its node says is now reported as the node's raw components (`VcardValue::Unknown`), so the report says what the merged card carries.

- A URI value was truncated at its first `;`, and escaped on the way back out.

  RFC 6350 section 4.2 gives a URI no structure and no escaping, but the codec read it as a structured value and kept only the first `;`-component, so `PHOTO:data:image/png;base64,AAAA` decoded to `data:image/png` and the payload was gone.

  Encoding then escaped the semicolon it had just used as a separator, so a value that did survive decoding did not survive its own round trip. A URI is now read whole and written back exactly as it is held, which is also what makes an inline data URI comparable between two sides of a merge.


- A phone number or an email address edited differently on both sides was kept twice instead of being reported as a collision.

  A property that may repeat and whose value names a thing outside the card is identified by that value, so the matching cannot see such a property change: it reads the edit as the old instance leaving and a new one arriving.

  Two arrivals then merged as a set, which is right for two additions and wrong for one instance edited two ways, and the card came back holding both numbers with nothing recorded against them.

  Two arrivals standing over one departure both sides agreed on are now a collision, resolved for the preferred side like any other, while two arrivals over nothing are still two additions and still merge as a set.

- Fixed the three-way merge losing a value edit past the first `;` or `,` of a non-structured value, silently.

  The merge decided whether two copies held the same value by comparing their *decoded* values, and that projection reads a value's first `;`-component alone, truncated again at its first unescaped `,`.

  Two divergent inline photos (`PHOTO:data:image/png;base64,...`), or two notes differing after a comma, produced no action at all: the change neither landed nor appeared in the report, so a caller resolving on an empty conflict report discarded one of them with nobody asked.

  Values now compare on the raw value node, component by component.

- Fixed a text value being truncated at an unescaped comma when decoded, so `NOTE:hello, world` no longer reads as `hello`.

- Fixed the line parser splitting a double-quoted parameter value that carries a `:` or a `;`, which RFC 6350 §3.3 allows and its own §6.3.1 `ADR` example uses.

  The head was cut at the first colon anywhere and parameters split on every semicolon, so that example parsed to one `GEO` parameter holding `"geo`, no `LABEL`, and an address shifted by one component.

  Both scans are now quote aware, falling back to the quote-blind scan when an unbalanced quote leaves no colon outside quotes, so a malformed line still parses.

- Fixed the merge reporting a card as disagreeing with itself when a property carries two parameters of one name, such as the `TEL;TYPE=work;TYPE=home` ordinary in vCard 2.1 and 3.0.

- Fixed the merge reporting a conflict for two sides adding one `TYPE` or `PID` set in two orders, which RFC 6350 §5.6 gives no order.

- Fixed the rule that an update beats a removal applying to a whole property only, so the outcome of a divergent parameter edit depended on which copy was passed as `left`. It now holds at parameter and list-parameter-item granularity too, and the number of reported conflicts no longer depends on the order of the two sides.

- Fixed a right-side list-item edit landing on a value the left side had replaced, producing a merged value neither side wrote and reporting nothing.

- Fixed an addition after a line a source file left unterminated being glued into that line's value, which destroyed the addition and could leave the merged card unparseable. Every line of the merged card but its last now carries a line ending.

- Fixed the merge treating a `BEGIN` or `END` line as an ordinary property, so a bare record's `END:VCARD` was replayed into a wrapped card and truncated it, and fixed an addition to a card carrying a vCard 2.1 `AGENT` landing inside the embedded card rather than beside the property it belongs to.

- Fixed a value replayed between cards of different versions carrying its escaping with it, so a 4.0 `NOTE:a\,c` arrived in a 2.1 card as a literal backslash.

  It is now re-encoded for the merged card's escaping mode, and two cards of different versions compare their values on the raw bytes, since they share no decoding: `URL:http\://x` at two versions is one value, not a change.

- Fixed a line break written into a vCard 2.1 value being emitted raw, which ended the content line and left a card that no longer parsed. vCard 2.1 has no line-break escape, so the writer now emits `\n`; the reader still resolves `\;` alone, as before.

- Fixed the merge replaying a list-parameter item as its decoded text, so a `TYPE=a\nb` landed as a real line break in the middle of the line's head and the merged card no longer parsed. The item is now written as the right card spelled it.

- Fixed the merge losing a line nobody touched when a card carries two instances of one property name under one `PID`: the untouched instance was paired with the edited one and deleted. Instance matching now pairs on `PID` and equality together before either alone.

- Fixed the merge dropping two copies of a repeated list item when both sides dropped one, so `NICKNAME:a,a` trimmed to `NICKNAME:a` on both sides merged to an empty value.

- Fixed the merge removing an interchangeable duplicate the other copies carried byte for byte, keeping instead a copy spelled differently. Instance matching now prefers identical bytes over decoded equality.

- Fixed the parser keeping the leftover whitespace of a continuation that follows a whitespace-only line, which named a property `" A"` instead of `A` and left a card that did not reparse to itself.

## [0.2.1] - 2026-08-21

### Fixed

- Fixed the JSContact export tagging every URI-valued resource object with a pre-RFC draft type name.

  `MediaResource`, `CryptoResource`, `CalendarResource`, `LinkResource` and `DirectoryResource` are now `Media`, `CryptoKey`, `Calendar`, `Link` and `Directory`, as RFC 9553 §2.6 registers them; a strict server rejected the draft spelling, so a contact carrying so much as a `URL` could not be written over JMAP.

  Import still ignores `@type`, so an earlier Card converts back unchanged.

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

  It diffs two divergent edits against their common base into per-side `VcardMergeAction` lists, matching property instances by PID then equality then position, and replays the right side onto a clone of the left byte-preservingly.

  A divergent same-field change becomes a `VcardMergeConflict`, where the left action wins except an update over a removal.

- Added opt-in content-decoding features, each backed by a `no_std` crate.

  `quoted-printable` decodes `=XX` octets, `base64` decodes inline binary values, and `encoding` transcodes a foreign `CHARSET`. The core keeps such values raw and their parameters intact, so nothing is silently transcoded.

- Added the RFC 7095 jCard codec behind the `jcard` feature (off by default, requires `parser`).

  `Vcard::to_jcard` and `from_jcard` project the decoded model to and from a `serde_json::Value`, resolving value kinds through the property specs. Export follows the RFC; import accepts anything structurally sound.

- Added the RFC 9554 vocabulary, modeled first-class.

  The `CREATED`, `GRAMGENDER`, `LANGUAGE`, `PRONOUNS` and `SOCIALPROFILE` properties and the nine new parameters (plus the RFC 9555 `JSPROP` and `JSPTR`) each gain a lens marker and spec, and the property-agnostic ones are allowed on any 4.0 property.

  `VcardAdr` carries all eighteen address components, writing the extended slots only when one is filled.

- Added the RFC 9555 JSContact conversion behind the `jscontact` feature (off by default, requires `jcard`).

  `Vcard::to_jscontact` and `from_jscontact` convert to and from an RFC 9553 Card, infallibly aside from a non-object import root. Unmappable properties are preserved in `vCardProps`, leftover parameters in `vCardParams`, and unknown Card members as `JSPROP` properties.

[unreleased]: https://github.com/pimalaya/vcard/compare/v0.2.1..HEAD
[0.2.1]: https://github.com/pimalaya/vcard/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/vcard/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/vcard/compare/root..v0.1.0
