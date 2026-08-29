---
cairn: delta
change: byte-preservation-needs-distinguishable-instances
---

## ADDED Requirements

### Requirement: Byte preservation needs distinguishable instances

An untouched line SHALL keep its exact bytes through a merge whenever its instance can be told apart from the card's others. A card carrying the same property twice with the same content leaves the three matchings free to pair the copies differently, so which copy a removal takes, and which spelling survives, is not promised; the content is, by the completeness law.

#### Scenario: One property carried twice, identically
- GIVEN a card carrying one property twice with the same content and one copy removed on one side
- WHEN they are merged
- THEN one copy survives, and which of the two spellings it has is not pinned

## MODIFIED Requirements

## REMOVED Requirements
