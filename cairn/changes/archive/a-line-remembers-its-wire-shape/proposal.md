---
cairn: change
id: a-line-remembers-its-wire-shape
status: landed
created: 2026-08-29
---

# A line remembers how it was laid out on the wire

## Why

The crate's value proposition is a byte-faithful CST, and folding is the one place it is not. A line is unfolded on parse and nothing remembers it was folded, so a card exported by Apple, iOS or Google (all of which fold heavily) comes back rewritten. The same goes for a QUOTED-PRINTABLE soft break and for a blank line between properties.

Two consumers pay for it today. tcard retired its own byte-preserving editor onto this crate, so an untouched project-then-apply round trip now rewrites every folded line of the card. neverest merges contact bodies through `VcardCst` and pushes the result to CardDAV, so a one-field edit rewrites the whole body: every push is larger than it needs to be, and every other client sees the entire card change rather than the field that did.

The corpus round-trip laws pass today only because they assert a fixpoint rather than byte identity, and the proptest generator feeds the merge laws mostly clean input.

## What

Give the line the memory ical-rs already gives it. A new `tree::wire::VcardWire` records, against the line's *logical* bytes, every piece the tokeniser resolved away: a fold (the break and the single whitespace apart, so `\n\t` does not come back as `\r\n `), a QUOTED-PRINTABLE soft break, and a run of bytes dropped verbatim (the blank lines before a line, the whitespace of a dangling continuation, a dangling `=`). Serialization re-inserts them. The logical length is sealed with the offsets and a stale shape is dropped, so an edited line goes out unfolded rather than folded in the wrong places.

Blank lines *after* the last line have nowhere to live, so `VcardCst` gains a `trailing` field, and the card-level leading-blank-line trim goes, since the tokeniser now records what it skips.

The two crates are deliberate twins, so the shape, the naming and the file layout follow ical-rs, and any divergence is stated. The corpus laws move from fixpoint to byte identity, and the merge generator feeds folded, soft-broken and blank-lined input.
