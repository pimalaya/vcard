---
cairn: delta
change: update-beats-a-parameter-removal
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: The merge obeys its algebraic laws

`merge(base, x, x)` SHALL yield `x` byte for byte and report nothing, `merge(base, x, base)` SHALL yield `x` byte for byte, `merge(base, base, y)` SHALL yield `y`, and `merge(base, base, base)` SHALL yield the base. The merged card SHALL parse again to a byte-stable fixpoint. Swapping the two sides SHALL report the same collided fields, and as many conflicts, though the merged bytes differ, since the left action wins. Re-merging the merged card against the base and either side SHALL change nothing.

#### Scenario: An untouched side
- GIVEN a base card and one copy that changed nothing
- WHEN they are merged in either order
- THEN the merged card is the other copy and nothing is reported

### Requirement: Conflicts are reported, never silently resolved away

A divergent change to the same field on both sides SHALL be surfaced as a `VcardMergeConflict` in the `VcardMergeReport`, with the left action winning, except that an update beats a removal, at every granularity the merge diffs at: the whole property, one parameter, and one item of a list parameter alike.

A parameter an update restores over a removal is appended to the merged line, since the removal took its position with it and parameter order carries no meaning.

#### Scenario: Update against removal
- GIVEN a base property that the left copy removes and the right copy updates
- WHEN they are merged
- THEN the update wins and the conflict is reported

#### Scenario: A parameter update against a parameter removal
- GIVEN a base `TEL;PREF=1`, one copy dropping `PREF` and the other rewriting it to `PREF=2`
- WHEN they are merged in either order
- THEN the merged card carries `PREF=2` and one conflict is reported

## REMOVED Requirements
