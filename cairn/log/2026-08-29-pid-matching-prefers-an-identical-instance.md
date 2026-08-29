---
cairn: log
change: pid-matching-prefers-an-identical-instance
landed: 2026-08-29
---

# Two instances under one PID pair by equality first

Found by the merge fuzz target: a line all three copies carried was gone from the merged card.

`PID` is instance identity and the matching pairs by it first, so a card carrying two instances of one name under one `PID`, which real cards do, made the overlap test true for both candidates and the greedy pass paired them in source order. That broke the pair needing no change at all: the untouched instance was matched with the edited one, so the merge saw one value change and one removal, rewrote the first line and deleted the second.

The matching now runs a first pass requiring both `PID` overlap and equality. With a distinct `PID` per instance that pass is a subset of the next one and pairs exactly what it used to, so nothing else moves.

Spec updated: `merge` (MODIFIED: instance matching is PID, then equality, then position).
