---
cairn: change
id: parameter-agreement-is-a-set
status: landed
created: 2026-08-29
---

# The whole-parameter path honours identity and set semantics

## Why

Two defects, one cause: the replay decides whether the two sides agree by looking at *one* left action on the collided slot and comparing it with `==`.

`diff_params` compares the two sides' parameters of one key index by index, so `TEL;TYPE=work;TYPE=home` yields two actions on the one slot `Param("TYPE")`. `colliding()` returns the first of them, so the right side's *second* action is compared against the left side's *first*, they differ, and a conflict is recorded for a change both sides made identically. `merge(base, x, x)` then reports a disagreement between a card and itself. `TEL;TYPE=WORK;TYPE=VOICE` is ordinary vCard 2.1 and 3.0 and the corpus carries it.

RFC 6350 section 5.6 also gives `TYPE` a comma-separated list of type values with no ordering, and the merge agrees as long as the base carries the parameter, since the single-old-single-new path then diffs the items. When the base does not carry it, both sides' additions are whole `ParamAdded` actions compared with `==`, which is order-sensitive on the `Vec` inside `VcardParam::Type`, so two sides adding one set in two orders are reported as disagreeing. `PID` behaves the same way.

Neither loses data: the merged card is the left clone and already holds the change. Both make a synchronisation engine surface a conflict to a person for two replicas that agree.

## What

Before looking for a colliding left action, the replay asks whether *any* left action on the same base instance is the same change, and treats that as agreement: nothing to replay and nothing to report. Sameness is action equality, except that a list parameter's items compare as a set, since they carry no order.

The whole-parameter arms and the structured-component arm all go through it. The whole-value arm keeps its own node comparison, which is stronger.

## Judgement calls, for review

**A list parameter's items compare sorted, not case-folded.** RFC 6350 makes the type values case-insensitive, but the merge nowhere else folds case, and folding it here would silently rewrite what a side wrote. `TYPE=WORK` against `TYPE=work` therefore stays a disagreement, which is reported, not resolved.

**Agreement is per instance, not per slot ordinal.** A repeated parameter name has no identity beyond its key, so with two `TYPE` parameters that the two sides edited only partly alike, the residual left action can still collide with an unrelated right one. That is reported, never silently resolved, and giving a repeated parameter a positional identity would make it shift under a removal. The narrower defect, a side disagreeing with itself, is what this change removes.
