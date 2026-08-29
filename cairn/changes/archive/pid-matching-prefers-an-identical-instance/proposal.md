---
cairn: change
id: pid-matching-prefers-an-identical-instance
status: landed
created: 2026-08-29
---

# Two instances under one PID pair by equality first

## Why

Found by the merge fuzz target: a line all three copies carried was gone from the merged card.

`PID` is instance identity, and the matching pairs by it first. When a card carries two instances of one name under *one* `PID`, which real cards do, the overlap test is true for both candidates and the greedy pass pairs them in source order. That breaks the pair that needs no change at all:

    base   TEL;PID=1.1:+1   TEL;PID=1.1:+2
    right  TEL;PID=1.1:+3   TEL;PID=1.1:+2

The untouched `+2` is matched with the edited `+3`, so the merge sees one value change and one removal, rewrites the first line and deletes the second, and `TEL;PID=1.1:+2` disappears although nobody touched it.

## What

The matching gains a first pass that requires both `PID` overlap and equality, before the passes for each alone. Among the candidates that share an identity, an unchanged instance is matched with its unchanged counterpart, so the merge sees no change where there is none.

The documented order becomes: `PID` and equality, then `PID`, then equality, then position. Nothing else moves: with a distinct `PID` per instance, the first pass is a subset of the second and pairs exactly what it used to.
