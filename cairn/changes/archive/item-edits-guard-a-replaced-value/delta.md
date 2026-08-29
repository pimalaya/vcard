---
cairn: delta
change: item-edits-guard-a-replaced-value
---

## ADDED Requirements

### Requirement: A replaced value outranks an item edit

A right-side item edit of a list value the left side replaced as a whole SHALL be reported as a conflict and dropped, so the merged value is the left side's replacement rather than a hybrid neither side wrote. Two item edits still merge as a set and never collide.

#### Scenario: An item added to a replaced list
- GIVEN a base `CATEGORIES:a,b`, a left copy replacing the whole value and a right copy adding one item
- WHEN they are merged
- THEN the merged value is the left copy's, and the collision is reported

## MODIFIED Requirements

## REMOVED Requirements
