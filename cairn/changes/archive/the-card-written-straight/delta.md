---
cairn: delta
change: the-card-written-straight
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Structured values get bespoke types

A value whose grammar is genuinely structured SHALL have its own type rather than a generic component list: `VcardN`, `VcardAdr`, `VcardGender`, `VcardOrg`, `VcardGeo`, `VcardClientPidMap`, `VcardBinary`, `VcardDateAndOrTime`, `VcardTimestamp`, `VcardUtcOffset`, `VcardText`, `VcardTextList`, `VcardUri` and `VcardLanguageTag`.

`VcardAdr` carries the full eighteen address components, writing the eleven RFC 9554 extended slots only when one is filled. `VcardValue::Adr` SHALL hold it boxed: its eighteen lists inline would make every value, and so every property, about three times its size (432 and 504 bytes against 120 and 192).

#### Scenario: An extended address
- GIVEN a 4.0 `ADR` with only the seven classic components filled
- WHEN it is encoded
- THEN the eleven extended slots are not written

## REMOVED Requirements
