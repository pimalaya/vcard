---
cairn: spec
capability: decoded-model
status: current
---

# Decoded model

The semantic side of the crate: `Vcard`, `VcardVersion`, `VcardProp`, `VcardParam`, `VcardValue` and the structured value types. It is pure data, available without the `parser` feature, so a consumer that only needs the model can depend on it alone.

A single version-agnostic model reads and writes vCard 2.1, 3.0 (RFC 2426) and 4.0 (RFC 6350) alike. A `Vcard` is a version plus a list of `VcardProp`, each a name, its parameters and one value.

### Requirement: No dependency on the syntax side

The decoded model SHALL carry no wire name, no escaping and no dependency on the `tree` module, so it compiles with `--no-default-features` and pulls in no crates at all.

The property name lives on `VcardProp::name`; escaping and framing live on the syntax side (see [parsing](./parsing.md)).

#### Scenario: The bare core
- GIVEN the crate built with `--no-default-features`
- WHEN the dependency graph is inspected
- THEN it contains no crate beyond `core` and `alloc`

### Requirement: Closed vocabularies reached through FromStr and Deref

Property names, parameter names, value types and versions SHALL each be a closed, fieldless enum (`VcardPropKind`, `VcardParamKind`, `VcardValueKind`, `VcardVersion`) whose wire spelling is reached through `Deref<Target = str>` and whose parsing is `FromStr`.

Parsing a name is case-insensitive. `VcardPropKind::ALL` enumerates every known property.

#### Scenario: A lowercase wire name
- GIVEN the wire name `fn`
- WHEN it is parsed into a `VcardPropKind`
- THEN it yields the `Fn` variant, whose `Deref` is `FN`

### Requirement: Open payloads keep unmodelled data

`VcardParam` and `VcardValue` SHALL each carry an `Unknown` arm holding the raw name and components, so a parameter, value type or property outside the model survives a decode and re-encode.

`VcardPropName` holds either a known `VcardPropKind` or a verbatim unknown name. An unrecognised or missing card version normalises to `VcardVersion::V4_0` in the decoded model, while byte-faithful round-tripping stays on the syntax tree.

A text value keeps its whole first component, unescaped: a comma must be escaped inside a text value, so an unescaped one is content rather than a separator and never truncates the value.

#### Scenario: A vendor extension
- GIVEN a card carrying an `X-VENDOR-THING` property with an `X-FLAG` parameter
- WHEN it is decoded and re-encoded
- THEN both the property and the parameter come back with their original spelling and values

#### Scenario: An unescaped comma in a note
- GIVEN a card carrying `NOTE:hello, world`
- WHEN it is decoded
- THEN the text reads `hello, world` rather than stopping at the comma

### Requirement: Structured values get bespoke types

A value whose grammar is genuinely structured SHALL have its own type rather than a generic component list: `VcardN`, `VcardAdr`, `VcardGender`, `VcardOrg`, `VcardGeo`, `VcardClientPidMap`, `VcardBinary`, `VcardDateAndOrTime`, `VcardTimestamp`, `VcardUtcOffset`, `VcardText`, `VcardTextList`, `VcardUri` and `VcardLanguageTag`.

`VcardAdr` carries the full eighteen address components, writing the eleven RFC 9554 extended slots only when one is filled.

#### Scenario: An extended address
- GIVEN a 4.0 `ADR` with only the seven classic components filled
- WHEN it is encoded
- THEN the eleven extended slots are not written
