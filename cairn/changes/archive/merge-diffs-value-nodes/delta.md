---
cairn: delta
change: merge-diffs-value-nodes
---

## ADDED Requirements

### Requirement: Values are compared on the raw node

The merge SHALL decide whether two copies hold the same value by comparing their raw value nodes component by component, over every component of the value, never through the decoded projection, which reads only a value's first `;`-component.

Two components agree when they decode to the same list of items, so a difference in escaping is a difference. An absent component and an all-empty one agree, so a trailing empty component is not a change.

#### Scenario: A photo payload past the first semicolon
- GIVEN a base card carrying `PHOTO:data:image/png;base64,AAAA` and a copy carrying a different payload
- WHEN they are merged
- THEN the change is reported and lands, and two divergent payloads collide

## MODIFIED Requirements

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

## REMOVED Requirements
