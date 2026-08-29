---
cairn: change
id: item-edits-guard-a-replaced-value
status: landed
created: 2026-08-29
---

# A right list-item edit must not land on a value the left side replaced

## Why

`Slot::collides_with` pairs a left item edit with a right whole-value change but not the reverse: there is no `(Value, Items)` arm. The `ValueItemAdded` and `ValueItemRemoved` arms of the replay also run no collision check at all, unlike the component and parameter arms. So when the left side replaces the whole value and the right side only adds an item to the list it still sees, the right item is appended to the left side's replacement:

    base   CATEGORIES:a,b
    left   CATEGORIES;VALUE=text:x
    right  CATEGORIES:a,b,c

    merged CATEGORIES;VALUE=text:x,c    with no conflict reported

`x,c` is not the base, not the left value and not the right value, and nothing is reported. It takes a change of the value's decoded kind to reach, since two list values always diff item by item, which in practice means a `VALUE` parameter appearing, disappearing or changing.

## What

The missing arm, and the guard the neighbouring arms already carry: a right item edit whose value the left side replaced wholesale is reported as a conflict and dropped, the left replacement standing.

Item edits still never collide with each other, which is the set semantics the module documents.
