---
cairn: delta
change: a-value-change-reports-the-whole-value
---

## ADDED Requirements

### Requirement: A whole-value change reports the whole value

The `old` and `new` payloads of a `ValueChanged` action SHALL say what the two raw value nodes say. A value whose decoded projection does not encode back to what its node holds SHALL be reported as the node's raw components (`VcardValue::Unknown`), since a non-structured value decodes its first `;`-component alone and would otherwise be reported truncated, with its old and its new equal. A value the model reads whole SHALL keep its decoded kind.

#### Scenario: A note changed past its first semicolon
- GIVEN a base and a left copy holding `NOTE:a;b`, and a right copy holding `NOTE:a;CHANGED`
- WHEN they are merged
- THEN the reported change holds both values whole, and the merged card carries the new one

## MODIFIED Requirements

## REMOVED Requirements
