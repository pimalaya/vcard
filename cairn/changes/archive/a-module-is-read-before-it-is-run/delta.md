---
cairn: delta
change: a-module-is-read-before-it-is-run
---

## ADDED Requirements

(none)

## MODIFIED Requirements

### The strict layer is two modules

The spec-driven builder lives at `tree::builder` and the whole-card validation at `tree::validator`. The `tree::vcard` module that held both is gone; `VcardBuilder`, `VcardPropBuilder`, `VcardValid` and `VcardValidateError` are otherwise unchanged.

## REMOVED Requirements

(none)
