---
cairn: delta
change: ours-wins
---

## MODIFIED Requirements

### Requirement: Conflicts are reported, never silently resolved away

A divergent change to the same field on both sides SHALL be surfaced as a `VcardMergeConflict` in the `VcardMergeReport`, with the left side's action winning, except that an update beats a removal, at every granularity the merge diffs at: the whole property, one parameter, and one item of a list parameter alike.

A parameter an update restores over a removal is appended to the merged line, since the removal took its position with it and parameter order carries no meaning.

#### Scenario: Update against removal
- GIVEN a base property that the left copy removes and the right copy updates
- WHEN they are merged
- THEN the update wins and the conflict is reported

#### Scenario: A parameter update against a parameter removal
- GIVEN a base `TEL;PREF=1`, one copy dropping `PREF` and the other rewriting it to `PREF=2`
- WHEN they are merged in either order
- THEN the merged card carries `PREF=2` and one conflict is reported

### Requirement: Ours wins, and the collision is still reported

The left side SHALL be `ours` and the right side `theirs`, in git's sense. The merged card SHALL be built from the left side's bytes, and where both sides changed one field to different things it SHALL carry the left side's value. Neither is a caller's to choose.

One side answers both questions on purpose. A caller reaches for a merge holding the version it is merging into, and that version is the one it would rather not churn and the one it means to keep. Every collision is reported either way, so a caller wanting the other value resolves it from the report rather than asking the merge to guess.

The rule SHALL decide only the case where both sides wrote a value. An update still beats a removal whichever side it came from, a field one side alone touched is still taken from that side, an untouched line still comes out byte for byte, and the report still names both actions and the same fields.

A parameter or a property that loses SHALL NOT be written beside the one that beat it, so a name a version allows at most once is never written twice.

#### Scenario: A field one side alone touched
- GIVEN a field only the left copy changed, beside a field both copies changed
- WHEN they are merged
- THEN the left copy's change survives and nothing is reported for it

#### Scenario: A parameter both sides rewrote
- GIVEN a base `TEL;PREF=1`, a left copy holding `PREF=2` and a right copy holding `PREF=3`
- WHEN they are merged
- THEN the merged card carries `PREF=2` alone and the collision is reported

#### Scenario: Two additions of a name allowed at most once
- GIVEN a base without `UID`, and two copies each adding a different one
- WHEN they are merged
- THEN the merged card carries the left copy's `UID` alone and the collision is reported

### Requirement: The merge obeys its algebraic laws

`merge(base, x, x)` SHALL yield `x` byte for byte and report nothing, `merge(base, x, base)` SHALL yield `x` byte for byte, `merge(base, base, y)` SHALL yield `y`, and `merge(base, base, base)` SHALL yield the base. The merged card SHALL parse again to a byte-stable fixpoint. Swapping the two sides SHALL report the same collided fields, and as many conflicts, though the merged bytes differ, since the left side's action wins. Re-merging the merged card against the base and either side SHALL change nothing.

#### Scenario: An untouched side
- GIVEN a base card and one copy that changed nothing
- WHEN they are merged in either order
- THEN the merged card is the other copy and nothing is reported

## REMOVED Requirements

### Requirement: The winning side is chosen, not implied

Removed. A caller can no longer say which side wins a collision. The rule is the convention git already names, and the split it replaced was justified by a distinction no caller ever drew: every one of them passed the left side.
