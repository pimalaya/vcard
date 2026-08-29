---
cairn: delta
change: matching-prefers-identical-bytes
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Instance matching is PID, then equality, then position

A property instance SHALL be matched across the three copies by its `PID` parameter and equality together first, then by `PID` alone (the RFC 6350 section 7 synchronisation identity), then by identical serialized bytes, then by equality alone, then by position, so a card that carries synchronisation identifiers merges by identity rather than by order, two instances sharing one `PID` do not break the pair that needs no change, and among interchangeable instances the one that needs no rewrite is chosen.

#### Scenario: Reordered instances
- GIVEN two copies whose `TEL` instances carry `PID` and appear in different orders
- WHEN they are merged
- THEN each instance is matched by its `PID`, not by its position

#### Scenario: Two instances under one PID
- GIVEN a base carrying two `TEL` instances under one `PID`, and a copy that edited one of them
- WHEN they are merged
- THEN the untouched instance keeps its bytes and only the edited one changes

#### Scenario: An interchangeable duplicate spelled two ways
- GIVEN a base carrying one property three times, one of them with a different line ending, and a copy carrying only the two identical ones
- WHEN they are merged
- THEN the copy the other two carry byte for byte survives

## REMOVED Requirements
