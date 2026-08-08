---
cairn: spec
capability: merge
status: current
---

# Three-way merge

`tree::merge::merge` reconciles two divergent copies of a card against their common base, building on the byte-preserving edit layer (see [editing](./editing.md)) so an untouched property keeps its bytes through a merge.

### Requirement: Per-side action lists against a common base

The merge SHALL diff each side against the base into a list of `VcardMergeAction`, then replay the right side's actions onto a clone of the left.

#### Scenario: Disjoint edits
- GIVEN a base card, a left copy editing `FN` and a right copy editing `EMAIL`
- WHEN they are merged
- THEN the result carries both edits and no conflict

### Requirement: Instance matching is PID, then equality, then position

A property instance SHALL be matched across the three copies by its `PID` parameter first, then by value equality, then by position, so a card that carries synchronisation identifiers merges by identity rather than by order.

#### Scenario: Reordered instances
- GIVEN two copies whose `TEL` instances carry `PID` and appear in different orders
- WHEN they are merged
- THEN each instance is matched by its `PID`, not by its position

### Requirement: Conflicts are reported, never silently resolved away

A divergent change to the same field on both sides SHALL be surfaced as a `VcardMergeConflict` in the `VcardMergeReport`, with the left action winning, except that an update beats a removal.

#### Scenario: Update against removal
- GIVEN a base property that the left copy removes and the right copy updates
- WHEN they are merged
- THEN the update wins and the conflict is reported
