---
cairn: spec
capability: conformance
status: current
---

# Conformance

The "strict out" half of the crate: the per-property spec layer, whole-card validation and the spec-driven builder. All three read one source of truth, so a rule is stated once.

The builder lives at `builder` and the validation at `validator`, both at the crate root. The per-property contract is `prop::spec`, with the `prop::cardinality` multiplicity axis, and each property marker is defined at `prop::<name>` carrying its `VcardPropSpec` impl. The marker's lens half, its edit cursor and any bespoke codec stay at `tree::prop::<name>`.

All of it works with the `parser` feature off: a spec is a statement about the RFC rather than about bytes, so validating, building and converting a decoded card compile no tokeniser.

Validity and lossiness are orthogonal. A conformant card may still carry `X-` or IANA extensions, so a no-`Unknown` "strict" model type would mean "no extensions", a useless category. Validity is therefore a runtime predicate, not a second data model.

### Requirement: One spec per property, three readers

Each property SHALL carry a `VcardPropSpec` on its marker declaring the versions it lives in, its `VcardPropCardinality`, the value kinds and parameters it may take per version, and the value kind in force given a declared `VALUE`.

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

### Requirement: A closed value vocabulary is validated
Where a definition closes the content of a value, `validate` SHALL check it and report what it found. A property declares that through `VcardPropSpec::invalid_value`, which defaults to allowing everything: the rest of the spec describes a property's shape, and this member alone describes what may be inside its value.

`GENDER`'s sex component SHALL be one of `M`, `F`, `O`, `N`, `U` or empty (RFC 6350 6.2.7), `PROFILE`'s value SHALL be `VCARD` (RFC 2426 3.6.3), and `CLIENTPIDMAP`'s first field SHALL be a positive integer (RFC 6350 6.7.7). Matching SHALL be case-insensitive, RFC 5234 making a quoted ABNF literal so.

A vocabulary whose grammar ends in `iana-token / x-name` is open and SHALL NOT be checked. `KIND`, `CLASS`, `GRAMGENDER`, `CALSCALE`, `PHONETIC` and every `TYPE` set are open in exactly that way, and rejecting a value outside their listed ones would refuse cards that conform.

### Requirement: A constrained parameter value is validated
`PREF` SHALL be an integer from 1 to 100 (RFC 6350 5.3), `PID` one or more digits optionally followed by a dot and more digits (5.5), and `DERIVED` either `true` or `false` (RFC 9554 3.4). Those constraints do not vary by the property carrying the parameter, so one check SHALL serve every appearance rather than each property restating it.

### Requirement: Content validation is not format validation
What `validate` checks inside a value SHALL be closed vocabularies and small integers. Dates, URIs, UTC offsets and language tags have grammars too, and checking them is a different undertaking with a far fuzzier edge: the crate would carry a URI parser to answer a question no caller asked.

Reading SHALL stay maximally liberal either way. A card carrying a value outside its vocabulary SHALL still parse and still round-trip byte for byte, and the decoded value SHALL stay the open type it is rather than becoming an enum. Strictness lives in `validate`, which is where a caller goes to ask.
