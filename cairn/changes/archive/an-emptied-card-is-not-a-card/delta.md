---
cairn: delta
change: an-emptied-card-is-not-a-card
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: The merged card is a card

Every line of the merged card but its last SHALL carry a line ending, so an addition after a line a source file left unterminated stays its own line and the merged card parses back to itself.

A line whose ending is already there keeps it verbatim, and a line that is still last keeps its empty one.

A merge that removes every line of a card yields no bytes at all, which the parser does not read back: an empty document is not a card, and the merge does not invent one to keep the fixpoint.

#### Scenario: An addition after an unterminated record
- GIVEN a base record `FN:a\r\nNOTE:b` read without a trailing break, and a right copy adding a `TEL`
- WHEN they are merged
- THEN the merged record holds three lines and reparses to three properties

#### Scenario: A record whose every property is removed
- GIVEN a bare record carrying one property and a right copy carrying only an `END` line
- WHEN they are merged
- THEN the merged record is empty, and the reparse law does not apply to it

## REMOVED Requirements
