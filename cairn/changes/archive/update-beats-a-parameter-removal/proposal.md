---
cairn: change
id: update-beats-a-parameter-removal
status: landed
created: 2026-08-29
---

# An update beats a removal at parameter granularity too

## Why

cairn/spec/merge.md says a divergent change is surfaced "with the left action winning, except that an update beats a removal", and the module header repeats it as "except when a removal meets an update, where the update wins (data survives over silent loss)". Neither restricts the exception to whole properties, and the module lists one parameter among the granularities it diffs at.

At property granularity the exception holds on both sides. At parameter granularity there is no such arm, so the left action wins whichever it is:

    base    TEL;PREF=1:+1
    removed TEL:+1
    updated TEL;PREF=2:+1

    merge(base, updated, removed) -> TEL;PREF=2:+1   the update survives
    merge(base, removed, updated) -> TEL:+1          the update is dropped

The same asymmetry reaches a list parameter, where one side dropping `TYPE` and the other adding an item keeps `TYPE=work,cell` one way round and nothing the other.

Nothing is silent: a conflict is reported both ways. But the outcome depends on which replica the caller happens to pass as `left`, which is exactly what the rule exists to prevent, and a caller that trusts the documented rule resolves conflicts wrongly.

## What

The parameter arms of the replay grow the exception the property arms already have. When the left action on the collided parameter is a removal and the right action is an update, the conflict is still reported and the update lands: the right side's parameter node is written onto the merged line, appended when the left side had removed it outright.

A right item edit whose parameter the left side removed restores that whole parameter from the right card rather than reporting and dropping the edit, which is the same rule at item granularity.

## Judgement calls, for review

**The exception grows rather than the rule shrinking.** The alternative is to write down that the exception is property-level only, which is defensible (a parameter is metadata, not content). It was rejected because the two are indistinguishable to a caller resolving a conflict: whichever way the rule reads, the merge must not depend on which copy is called left, and "data survives" is the rule the rest of the module already follows.

**A restored parameter is appended, not put back where it was.** The left side removed the node, so its position is gone; parameter order carries no meaning in any vCard version, and reconstructing it would rewrite bytes the left side deliberately wrote.

**Restoring a parameter also makes the conflict count symmetric.** A right item edit whose parameter is gone used to report one conflict per item, so swapping the sides changed the number of reported pairs. Restoring the parameter on the first such edit leaves the rest with nothing to collide against, so both directions now report one pair per collided parameter, and the law that compares the two directions is strengthened from the collided fields to the conflicts themselves.
