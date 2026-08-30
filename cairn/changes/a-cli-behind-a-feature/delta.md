---
cairn: delta
change: a-cli-behind-a-feature
---

## ADDED Requirements

### Requirement: The crate ships a CLI behind an off-by-default feature

The `cli` feature SHALL build a binary that parses, validates, converts and builds a card over the public API, adding no library surface of its own. Every data command SHALL print a documented output type, in a human spelling by default and in camelCase JSON under `--json`, and SHALL exit non-zero when the card it was given does not conform.

## MODIFIED Requirements

(none)

## REMOVED Requirements

(none)
