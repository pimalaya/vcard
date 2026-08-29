---
cairn: delta
change: structural-changes-run-last
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Per-side action lists against a common base

The merge SHALL diff each side against the base into a list of `VcardMergeAction`, then replay the right side's actions onto a clone of the left.

The replay SHALL run in two phases: every edit of a value, a component or a parameter happens in place while no line moves, and the structural changes follow, removals first on descending indices, then each addition placed against the card as it then stands. Nothing addressed by index is read after an index has moved.

#### Scenario: Disjoint edits
- GIVEN a base card, a left copy editing `FN` and a right copy editing `EMAIL`
- WHEN they are merged
- THEN the result carries both edits and no conflict

#### Scenario: A removal above an edit
- GIVEN a right copy that removes the card's first property and edits three that follow it
- WHEN they are merged
- THEN every edit lands on the property it names and nothing is reported

## REMOVED Requirements
