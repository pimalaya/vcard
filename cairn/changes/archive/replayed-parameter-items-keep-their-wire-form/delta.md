---
cairn: delta
change: replayed-parameter-items-keep-their-wire-form
---

## ADDED Requirements

### Requirement: A replayed parameter item keeps its wire form

An item the merge replays into a list parameter SHALL be written as the right card spelled it, never as its decoded text: a parameter value is unescaped on the way in and copied verbatim on the way out, so a decoded item carrying a line break would end the line in the middle of its head.

#### Scenario: A type value holding an escaped line break
- GIVEN a base `TEL;TYPE=work` and a right copy adding the item `a\nb`
- WHEN they are merged
- THEN the merged line reads `TEL;TYPE=work,a\nb` and parses back to itself

## MODIFIED Requirements

## REMOVED Requirements
