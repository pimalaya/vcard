---
cairn: change
id: a-merged-wire-shape-is-ordered-by-offset
status: landed
created: 2026-08-29
---

# A merged wire shape is ordered by offset

## Why

`VcardWire::prepend` merges the tokeniser's pieces with the line splitter's by a stable sort on the offset, because a value ending on two `=` gives the two of them one byte each at neighbouring offsets and the wrong order writes a line break into the value. That rule lives in the code and in the log of `a-line-remembers-its-wire-shape`, and nowhere in the spec.

ical-rs had the concatenation the rule replaced, and its own agent found the defect only by porting the shape. A rule that costs a byte-faithful round-trip when it is missing belongs in the spec of both twins, not in one crate's commit history.

## What

State the ordering rule in the parsing capability, beside the pieces it orders. No code moves: the behaviour is what the crate already does.
