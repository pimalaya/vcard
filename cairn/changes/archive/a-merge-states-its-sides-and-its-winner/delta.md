---
cairn: delta
change: a-merge-states-its-sides-and-its-winner
---

## ADDED Requirements
### Requirement: The winning side is chosen, not implied

A caller SHALL be able to say which side's value the merged card carries when both sides changed one field to different things, independently of which side supplies the baseline bytes. Where the caller says nothing, the left side wins, which is what a merge has always done.

The two are different questions. Which side is the baseline decides whose folding, whose parameter casing and whose property order survive untouched, and is answered by whichever copy the caller would rather not churn. Which side wins a collision is a statement about two people disagreeing, and is answered by what the caller knows about them. Deciding the second by the first makes a byte-fidelity choice settle a data-loss one.

The preference SHALL decide only the case where both sides wrote a value. An update still beats a removal whichever side it came from and whatever the preference, a field one side alone touched is still taken from that side, an untouched line still comes out byte for byte, and the report still names both actions and the same fields whichever way the preference falls.

A parameter or a property the preferred side's action beats SHALL be replaced rather than joined, so a name a version allows at most once is never written twice.

#### Scenario: The right side is preferred
- GIVEN two copies setting a different `FN`, and a caller preferring the right side
- WHEN they are merged
- THEN the merged card carries the right side's `FN` and the collision is reported as it always was

#### Scenario: The left preference stated is the preference left unsaid
- GIVEN any two copies and a caller stating the left side
- WHEN they are merged
- THEN the merged bytes and the report are what saying nothing gives

#### Scenario: The preference does not reach an uncontested field
- GIVEN a field only the left copy changed, and a caller preferring the right side
- WHEN they are merged
- THEN the left copy's change survives and nothing is reported for it

#### Scenario: An update beats a removal under either preference
- GIVEN one copy removing a property and the other changing it
- WHEN they are merged under either preference, in either order
- THEN the changed property survives and the collision is reported

## MODIFIED Requirements
### Requirement: Conflicts are reported, never silently resolved away

A divergent change to the same field on both sides SHALL be surfaced as a `VcardMergeConflict` in the `VcardMergeReport`, with the preferred side's action winning, the left one unless the caller says otherwise, except that an update beats a removal, at every granularity the merge diffs at: the whole property, one parameter, and one item of a list parameter alike.

A parameter an update restores over a removal is appended to the merged line, since the removal took its position with it and parameter order carries no meaning.

#### Scenario: Update against removal
- GIVEN a base property that the left copy removes and the right copy updates
- WHEN they are merged
- THEN the update wins and the conflict is reported

#### Scenario: A parameter update against a parameter removal
- GIVEN a base `TEL;PREF=1`, one copy dropping `PREF` and the other rewriting it to `PREF=2`
- WHEN they are merged in either order
- THEN the merged card carries `PREF=2` and one conflict is reported
