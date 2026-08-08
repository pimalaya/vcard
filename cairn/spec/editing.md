---
cairn: spec
capability: editing
status: current
---

# Editing

Reading and writing one property of a parsed card without disturbing the rest of it. The moving parts are the per-property lens markers in `tree::prop`, the per-parameter markers in `tree::param`, and the cursors in `tree::value::cursor`.

A lens marker is a zero-sized type-level key spelled exactly as its wire token (`FN`, `ADR`, `SORT_AS`), used as `card.prop::<FN>()`. It ties that token to a decoded value type, an edit cursor and the `decode` projection, and carries the property's spec (see [conformance](./conformance.md)).

### Requirement: Edits are byte-preserving

A setter SHALL rewrite only the component it touches, leaving every other leaf, every parameter and every other line of a parsed card byte for byte intact.

#### Scenario: One item of a list
- GIVEN a parsed list value whose first item carries a redundant escape such as `a\:b`
- WHEN a different item of the same list is removed
- THEN the redundant escape survives verbatim, because no whole-value rewrite happened

### Requirement: Cursors cover both the generic and the structured shapes

Scalar, list and URI properties SHALL share the generic `VcardValueCursor`, while the structured properties SHALL each carry a cursor naming their components: `VcardNCursor`, `VcardAdrCursor`, `VcardGenderCursor` and `VcardClientPidMapCursor`.

#### Scenario: A named component
- GIVEN a parsed `N` property
- WHEN its cursor's given-name accessor is set
- THEN only that component of the `N` value changes

### Requirement: A raw byte escape hatch

The value cursor SHALL expose raw byte accessors (`bytes` and `set_bytes`) beside its UTF-8 text accessors, so a value in a foreign character set can be read and written without transcoding.

#### Scenario: A Latin-1 value
- GIVEN a 2.1 card whose value declares `CHARSET=ISO-8859-1` and holds a non-UTF-8 octet
- WHEN the value is read through `bytes`
- THEN the raw octet comes back unaltered

### Requirement: Filling a required property

`VcardCst::fill_required` SHALL add a placeholder for a required property the card is missing for its version, leaving every existing line verbatim, and SHALL be idempotent.

#### Scenario: A 3.0 card with no N
- GIVEN a 3.0 card carrying `FN` but no `N`, which 3.0 makes mandatory
- WHEN `fill_required` runs twice
- THEN exactly one `N` line is added and the existing lines are untouched
