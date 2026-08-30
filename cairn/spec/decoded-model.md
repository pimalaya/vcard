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

A value the specification gives no `;`-structure of its own is decoded whole, unescaped: RFC 6350 section 3.4 has a text value escape a `;` or a `,` it means literally and section 4.2 gives a URI no escaping at all, so an unescaped separator is content rather than structure and never truncates the value. Only the structured kinds (`N`, `ADR`, `GENDER`, `ORG`, `GEO`, `CLIENTPIDMAP`) read component by component, and each of their components keeps the commas inside it.

#### Scenario: A vendor extension
- GIVEN a card carrying an `X-VENDOR-THING` property with an `X-FLAG` parameter
- WHEN it is decoded and re-encoded
- THEN both the property and the parameter come back with their original spelling and values

#### Scenario: An unescaped comma in a note
- GIVEN a card carrying `NOTE:hello, world`
- WHEN it is decoded
- THEN the text reads `hello, world` rather than stopping at the comma

#### Scenario: An unescaped semicolon in a note
- GIVEN a card carrying `NOTE:a;b`
- WHEN it is decoded
- THEN the text reads `a;b` rather than stopping at the semicolon

#### Scenario: A comma inside a structured component
- GIVEN a card carrying `CLIENTPIDMAP:1;urn:uuid:a,b`
- WHEN it is decoded
- THEN the client URI reads `urn:uuid:a,b` rather than stopping at the comma

### Requirement: A parameter value is encoded by RFC 6868, not by the text escapes

A parameter value SHALL be decoded and encoded by RFC 6868 section 3.1: `^n` reads as a newline, `^^` as a caret, `^'` as a double quote, and any other caret sequence, a trailing lone caret included, stays exactly as written, which section 3.1 requires rather than merely permits. A backslash SHALL be content in both directions, since RFC 6350 section 3.3 gives a parameter value no escapes at all and RFC 6868 section 3.2 forbids adding the backslash ones.

RFC 6868 updates RFC 6350 and no earlier specification, so the rules SHALL apply to vCard 4.0 alone. A 2.1 or 3.0 parameter carries its caret literally, and a parameter node SHALL therefore carry the escaping mode of the card it was parsed from, stamped once `VERSION` is known, as a value node already does. `VcardEscaper` SHALL name the three versions separately, 3.0 and 4.0 escaping a value identically but only 4.0 encoding a parameter.

A value the wire spelled inside its own double quotes SHALL keep that pair on the way out, only what they enclose being encoded. The decoded model holds a parameter exactly as it was written, delimiters included, so encoding the surrounding pair would strip the quoting off every quoted URI.

#### Scenario: The three sequences
- GIVEN `LABEL=a^nb^^c^'d` in a 4.0 card
- WHEN it is decoded and encoded again
- THEN it reads `a`, a newline, `b^c"d`, and comes back as `LABEL=a^nb^^c^'d`

#### Scenario: A caret before an ordinary letter
- GIVEN `LABEL=a^xb^`
- WHEN it is decoded
- THEN it reads `a^xb^`, the caret and what follows staying as written

#### Scenario: A backslash in a parameter
- GIVEN `X-PATH=C:\temp\note.txt`
- WHEN it is decoded
- THEN the value keeps both separators rather than losing them to a text escape

#### Scenario: A caret in a 3.0 parameter
- GIVEN `ADR;LABEL=a^nb` in a 3.0 card
- WHEN it is decoded
- THEN it reads `a^nb`, the version predating RFC 6868

#### Scenario: A quoted parameter through a round trip
- GIVEN `GEO="geo:37.386,-122.083"`
- WHEN it is decoded and encoded again
- THEN the bytes are the ones it arrived as

### Requirement: Structured values get bespoke types

A value whose grammar is genuinely structured SHALL have its own type rather than a generic component list: `VcardN`, `VcardAdr`, `VcardGender`, `VcardOrg`, `VcardGeo`, `VcardClientPidMap`, `VcardBinary`, `VcardDateAndOrTime`, `VcardTimestamp`, `VcardUtcOffset`, `VcardText`, `VcardTextList`, `VcardUri` and `VcardLanguageTag`.

`VcardAdr` carries the full eighteen address components, writing the eleven RFC 9554 extended slots only when one is filled.

#### Scenario: An extended address
- GIVEN a 4.0 `ADR` with only the seven classic components filled
- WHEN it is encoded
- THEN the eleven extended slots are not written
