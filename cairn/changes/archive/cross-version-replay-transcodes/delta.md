---
cairn: delta
change: cross-version-replay-transcodes
---

## ADDED Requirements

### Requirement: A replayed value carries its meaning, not its escaping

When the two cards escape values differently, which is vCard 2.1 against any later version, a value the merge replays from the right card SHALL be re-encoded for the merged card's escaping mode rather than copied byte for byte, so it keeps its meaning.

A value already written for the merged card's mode SHALL be copied unchanged, so a merge of one version's cards preserves its bytes.

Two cards escaping values by different rules share no decoding, so whether they hold the same value SHALL be decided on the raw bytes: the same line at two versions is not a change, and a line neither side touched is never rewritten.

#### Scenario: A 4.0 note replayed into a 2.1 card
- GIVEN a 2.1 base and left card carrying `NOTE:a,b` and a 4.0 right card carrying `NOTE:a\,c`
- WHEN they are merged
- THEN the merged 2.1 card carries the text `a,c`

## MODIFIED Requirements

## REMOVED Requirements
