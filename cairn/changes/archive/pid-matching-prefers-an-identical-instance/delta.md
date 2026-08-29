---
cairn: delta
change: pid-matching-prefers-an-identical-instance
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Instance matching is PID, then equality, then position

A property instance SHALL be matched across the three copies by its `PID` parameter and equality together first, then by `PID` alone (the RFC 6350 section 7 synchronisation identity), then by equality alone, then by position, so a card that carries synchronisation identifiers merges by identity rather than by order, and two instances sharing one `PID` do not break the pair that needs no change.

#### Scenario: Reordered instances
- GIVEN two copies whose `TEL` instances carry `PID` and appear in different orders
- WHEN they are merged
- THEN each instance is matched by its `PID`, not by its position

#### Scenario: Two instances under one PID
- GIVEN a base carrying two `TEL` instances under one `PID`, and a copy that edited one of them
- WHEN they are merged
- THEN the untouched instance keeps its bytes and only the edited one changes

## REMOVED Requirements
