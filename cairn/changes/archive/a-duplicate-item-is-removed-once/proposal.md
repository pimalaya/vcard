---
cairn: change
id: a-duplicate-item-is-removed-once
status: landed
created: 2026-08-29
---

# A removal both sides made takes one copy, not two

## Why

Found by the merge fuzz target: `merge(base, x, x)` did not yield `x`, which is the law that says two identical edits are not a disagreement.

An item removal is idempotent only while the list holds the item once. `TYPE=work,,` holds the empty item twice and `NICKNAME:a,a` holds `a` twice, both of which real cards carry. When both sides drop one copy, the merged card starts as the left clone, which is already one copy short, and the right side's identical removal takes a second copy that neither side wrote off:

    base   TEL;TYPE=work,,
    left   TEL;TYPE=work,
    right  TEL;TYPE=work,

    merged TEL;TYPE=work

The addition arms never had the problem, since they are presence-guarded.

## What

Both item-removal arms of the replay first ask whether the left side already made that removal, which is the check the whole-field arms carry, and do nothing when it did.

## Judgement call, for review

**A removal is matched by item, not by count.** When one side dropped one copy and the other dropped two, the merged card keeps two, not one: the left side's removal answers for the right side's first, and the second finds the same left action and stops. That follows the module's documented set semantics for list items, and the alternative, counting removals, would make a merge depend on how many duplicates a card happened to carry. The multiset-against-set tension it belongs to is already pinned by `duplicate_list_items_are_diffed_as_a_multiset_and_replayed_as_a_set`.
