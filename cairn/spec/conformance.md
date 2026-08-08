---
cairn: spec
capability: conformance
status: current
---

# Conformance

The "strict out" half of the crate: the per-property spec layer, whole-card validation, and the spec-driven builder. All three read one source of truth, so a rule is stated once.

Validity and lossiness are orthogonal. A conformant card may still carry `X-` or IANA extensions, so a no-`Unknown` "strict" model type would mean "no extensions", a useless category. Validity is therefore a runtime predicate, not a second data model.

### Requirement: One spec per property, three readers

Each property SHALL carry a `VcardPropSpec` on its lens marker declaring the versions it lives in, its `VcardPropCardinality`, the value kinds and parameters it may take per version, and the value kind in force given a declared `VALUE`.

A single vtable dispatch bridges the open `VcardPropKind` back to those static impls. The decoder consults it to pick a value kind, validation consults it to check conformance, and the builder consults it to reject illegal construction.

#### Scenario: A version-forked cardinality
- GIVEN the `FN` property, which RFC 6350 forks between versions
- WHEN its cardinality is asked for 3.0 and for 4.0
- THEN each version returns its own multiplicity

### Requirement: Validation mints a proof

`Vcard::validate` SHALL check per-version property existence, value kind, version-aware parameters and cardinality (including required-but-absent), while still permitting extensions, and SHALL return every violation at once rather than the first.

A card that passes earns `VcardValid<Vcard>`, a proof only validation can mint (through `validate` or its `TryFrom`). Both `Vcard` and `VcardValid<Vcard>` convert back into a `VcardCst`.

#### Scenario: A conformant card with an extension
- GIVEN a 4.0 card that satisfies RFC 6350 and also carries an `X-` property
- WHEN it is validated
- THEN it passes and yields the proof

#### Scenario: Several violations
- GIVEN a card breaking more than one rule
- WHEN it is validated
- THEN the error carries every violation, not just the first

### Requirement: The builder refuses illegal construction

`VcardPropBuilder` SHALL pin the property name and reuse the per-property check, rejecting a disallowed value kind or known parameter through `Result`, while still accepting extension parameters.

#### Scenario: A disallowed value kind
- GIVEN a builder pinned to a property whose spec allows only text
- WHEN a URI value is offered
- THEN the build returns an error rather than constructing the property

### Requirement: RFC 9554 parameters are property-agnostic

Validation SHALL allow the property-agnostic RFC 9554 parameters (`AUTHOR`, `AUTHOR-NAME`, `CREATED`, `DERIVED`, `PHONETIC`, `PROP-ID`, `SCRIPT`, `SERVICE-TYPE`, `USERNAME`) on any 4.0 property.

#### Scenario: PROP-ID on an arbitrary property
- GIVEN a 4.0 `EMAIL` property carrying `PROP-ID`
- WHEN the card is validated
- THEN the parameter is accepted
