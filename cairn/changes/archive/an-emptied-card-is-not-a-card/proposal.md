---
cairn: change
id: an-emptied-card-is-not-a-card
status: landed
created: 2026-08-29
---

# A merge that removes every line yields no card, and the law says so

## Why

Found by the merge fuzz target: the merged card failed to reparse with `MissingCrlf("")`, which is the parser's answer to no input at all.

A bare RFC 2425 record carrying only envelope lines has no properties, so merging it against a record of one property reads as "the right side removed it", the merge removes it, and the merged card serializes to zero bytes:

    base   FN:a
    right  END:VCARD

    merged (nothing)

The parser rejects an empty document, so the law that the merged card reparses to a byte-stable fixpoint fails on an outcome that is otherwise right: the right side really did have no properties.

## What

Nothing in the merge changes. The reparse law, in tests/merge.rs and in the fuzz target alike, no longer applies when the merged card has no bytes, and the merge capability says why: an empty document is not a card, and inventing a line to keep the output parseable would resurrect content nobody wrote.

`removing_every_line_leaves_no_card` pins the outcome, so a future change cannot start emitting a placeholder card unnoticed.

## Judgement call, for review

**The law is narrowed rather than the merge made to keep a line.** The alternatives are worse: emitting a `BEGIN` / `END` envelope the input never had invents structure, and refusing the last removal keeps content the right side deleted. A caller reading zero bytes back learns exactly what happened, and the case only arises for a bare record, since any card with an envelope or a `VERSION` line still has those lines.
