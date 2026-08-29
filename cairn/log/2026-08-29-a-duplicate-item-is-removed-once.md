---
cairn: log
change: a-duplicate-item-is-removed-once
landed: 2026-08-29
---

# A removal both sides made takes one copy, not two

Found by the merge fuzz target: `merge(base, x, x)` did not yield `x`.

An item removal is idempotent only while the list holds the item once, and `TYPE=work,,` and `NICKNAME:a,a` both hold one item twice. With both sides dropping one copy, the merged card started as the left clone, already one copy short, and the right side's identical removal took a second copy neither side wrote off.

Both item-removal arms now ask whether the left side already made that removal, the check the whole-field arms already carried.

Spec updated: `merge` (ADDED: a removal both sides made takes one copy).
