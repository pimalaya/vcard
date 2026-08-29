---
cairn: delta
change: merged-lines-are-terminated
---

## ADDED Requirements

### Requirement: The merged card is a card

Every line of the merged card but its last SHALL carry a line ending, so an addition after a line a source file left unterminated stays its own line and the merged card parses back to itself.

A line whose ending is already there keeps it verbatim, and a line that is still last keeps its empty one.

#### Scenario: An addition after an unterminated record
- GIVEN a base record `FN:a\r\nNOTE:b` read without a trailing break, and a right copy adding a `TEL`
- WHEN they are merged
- THEN the merged record holds three lines and reparses to three properties

## MODIFIED Requirements

## REMOVED Requirements
