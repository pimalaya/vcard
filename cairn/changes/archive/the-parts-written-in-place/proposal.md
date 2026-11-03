---
cairn: change
id: the-parts-written-in-place
status: landed
created: 2026-11-03
---

# The parts written in place

The third speed pass over the parser, the decoded model and the serializers, taken from the handoff [the-structure-scanned-once](../../the-structure-scanned-once/proposal.md) on 2026-11-03 and landed the same day. The handoff stays active, rewritten at the end of this pass for the next one.

## Why

The second pass ([the-card-written-straight](../the-card-written-straight/proposal.md)) left the parser and the decoder waiting rather than working: about 12 failed store forwards a line when parsing the short card and 18 when decoding it, each a multi-word part (the wire shape, the value `Cow`, a parameter's value list, the decoded value, a list header) built in one place by 8-byte stores and moved to another by 16-byte loads. Removing one copy at a time had only moved the wait to the next, five times over.

## What

No change to what any reader or writer returns, and no API change:

- The common content line (one physical line, not blank, indented, folded or soft-broken) is written into its slot field by field, every part in registers, its parameters and their value lists built where they land; the line end, the value colon and the name's semicolon are found sixteen bytes at a time over the rest of the input.
- A property is decoded into its slot the same way: a constant empty property grown in place, its value written by one store per kind, its item lists sized once and filled where they lie, a value with nothing to unescape stored from registers and the unescaping kept out of line.
- The property, parameter and value kind lookups match on the length and the first byte, then compare one candidate, the name never copied into a buffer.
- `to_bytes` writes a piece of sixteen bytes or fewer by fixed-size moves rather than a `memcpy` call, and the text writer writes a parameter value in place.

The log, cairn/log/2026-11-03-the-parts-written-in-place.md, gives every change with its cycles and the ones tried and dropped.
