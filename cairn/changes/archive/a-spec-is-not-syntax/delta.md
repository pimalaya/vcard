---
cairn: delta
change: a-spec-is-not-syntax
---

## ADDED Requirements

### Requirement: Conformance needs no parser

`Vcard::validate`, `VcardBuilder`, `VcardPropBuilder` and the jCard and JSContact codecs SHALL work with the `parser` feature off. The per-property spec they all read is a statement about the RFC rather than about bytes, so it SHALL live with the decoded model.

#### Scenario: A card is validated with no default features

- GIVEN a crate built with `--no-default-features`
- WHEN a decoded card is validated or built
- THEN it conforms or reports its violations, with no tokeniser compiled in

## MODIFIED Requirements

### The strict layer is at the crate root

The spec-driven builder lives at `builder` and the whole-card validation at `validator`, both at the crate root. The per-property contract is `prop::spec`, with the `prop::cardinality` multiplicity axis, and each property marker is defined at `prop::<name>` carrying its `VcardPropSpec` impl. The marker's `VcardPropLens` impl, its edit cursor and any bespoke codec stay at `tree::prop::<name>`.

## REMOVED Requirements

(none)
