---
cairn: delta
change: a-truncating-read-names-its-component
---

## ADDED Requirements

### Requirement: A truncating read names the component it truncates at

A value node SHALL read the whole value through readers that take no index (`decode`, `decode_list`, `decode_bytes`), keeping every `;` the value carries literal, and SHALL read one `;`-component only through readers that name it (`decode_component`, `decode_component_list`). No reader SHALL cut a value at both a `;` and a `,`.

The un-indexed writers (`set`, `set_bytes`) SHALL replace the whole value, so a value read whole and written back comes back unchanged. The component writers (`set_component`, `set_component_bytes`) SHALL rewrite nothing but the component they name.

The generic value cursor SHALL follow the same split: `text`, `bytes`, `list` and their setters address the whole value, `component` and `set_component` address one slot.

#### Scenario: A note read past its first semicolon
- GIVEN a card carrying `NOTE:a;b`
- WHEN the value is read through the generic cursor
- THEN it reads `a;b` rather than stopping at the semicolon

#### Scenario: A value read whole and written straight back
- GIVEN a value of several `;`-components
- WHEN it is read whole and written back through the un-indexed setter
- THEN it reads back as it went in, with no component of the old value left behind

## MODIFIED Requirements

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

### Requirement: Edits are byte-preserving

A setter that names a component SHALL rewrite only that component, leaving every other leaf, every parameter and every other line of a parsed card byte for byte intact. A setter that names none replaces the whole value, being the inverse of the reader that names none.

#### Scenario: One item of a list
- GIVEN a parsed list value whose first item carries a redundant escape such as `a\:b`
- WHEN a different item of the same list is removed
- THEN the redundant escape survives verbatim, because no whole-value rewrite happened

## REMOVED Requirements
