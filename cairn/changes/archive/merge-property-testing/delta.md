---
cairn: delta
change: merge-property-testing
---

# Delta

## ADDED Requirements

### Requirement: Every change either lands or is reported

For every field of the merged card, the merge SHALL leave what one side made it and that side changed it, or what the base held and neither side changed it. A side's change that did not land SHALL appear in the `VcardMergeReport`'s conflicts, and no field SHALL hold a value neither side wrote, except a set-valued field, which carries both sides' additions and removals.

The field granularities are the ones the merge diffs at: the whole property, the whole value of a non-structured property, one component of a structured value, the item set of a list value, one parameter, and the item set of a list parameter.

#### Scenario: A change that cannot land
- GIVEN a base card and two copies that changed one field differently
- WHEN they are merged
- THEN the merged field holds one side's value, and the field is named in the conflicts

### Requirement: The merge obeys its algebraic laws

`merge(base, x, x)` SHALL yield `x` byte for byte and report nothing, `merge(base, x, base)` SHALL yield `x` byte for byte, `merge(base, base, y)` SHALL yield `y`, and `merge(base, base, base)` SHALL yield the base. The merged card SHALL parse again to a byte-stable fixpoint. Swapping the two sides SHALL report the same set of collided fields, though the merged bytes differ, since the left action wins. Re-merging the merged card against the base and either side SHALL change nothing.

#### Scenario: An untouched side
- GIVEN a base card and one copy that changed nothing
- WHEN they are merged in either order
- THEN the merged card is the other copy and nothing is reported

## MODIFIED Requirements

## REMOVED Requirements
