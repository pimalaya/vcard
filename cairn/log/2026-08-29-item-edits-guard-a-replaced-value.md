---
cairn: log
change: item-edits-guard-a-replaced-value
landed: 2026-08-29
---

# A right list-item edit no longer lands on a value the left side replaced

`Slot::collides_with` had no `(Value, Items)` arm and the two item arms of the replay carried no collision guard at all, so a right-side item edit was appended to a value the left side had replaced wholesale. The merged `CATEGORIES;VALUE=text:x,c` was not the base, not the left value and not the right value, and nothing was reported.

The arm and the guard are in place: such an edit is reported and dropped, the left replacement standing. Two item edits still merge as a set and never collide.

Spec updated: `merge` (ADDED: a replaced value outranks an item edit).
