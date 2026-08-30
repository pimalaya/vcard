---
cairn: delta
change: two-parameters-of-one-name-are-two-fields
---

## ADDED Requirements

### Requirement: Each parameter occurrence is a field of its own

One parameter name may be written more than once on a property (`TEL;TYPE=work;TYPE=voice`, RFC 2426 section 4), and the field a parameter action occupies SHALL be addressed by the parameter's key and by its position among the property's parameters of that key. Two sides editing two different occurrences of one name SHALL both land, uncontested.

Each parameter action SHALL carry that position, so a caller reading the report can tell one occurrence from another, and the replay SHALL resolve the occurrence the action names rather than the first parameter of that name.

#### Scenario: Two sides editing two parameters of one name
- GIVEN a base `TEL;TYPE=work;TYPE=voice`, a left copy rewriting the first `TYPE` and a right copy rewriting the second
- WHEN they are merged
- THEN the merged card carries both edits and nothing is reported

## MODIFIED Requirements

## REMOVED Requirements
