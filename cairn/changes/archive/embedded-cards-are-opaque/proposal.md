---
cairn: change
id: embedded-cards-are-opaque
status: landed
created: 2026-08-29
---

# An embedded card owns its own lines

## Why

`VcardCst::props` holds real lines, envelope-looking ones included. A vCard 2.1 `AGENT` embeds a whole `BEGIN:VCARD`..`END:VCARD` that the parser keeps verbatim among the outer card's properties, and a bare RFC 2425 record has no envelope at all, so an `END:VCARD` line in one is an ordinary property. The merge skips only `VERSION`, so those lines are diffed, matched, moved and inserted around like any other. Two consequences, both silent.

An addition lands inside the embedded card, because `finish` places it after the last line sharing its name and, with an agent present, the last `FN` is the agent's:

    merged ...AGENT:/BEGIN:VCARD/VERSION:2.1/FN:Secretary/FN:Boss II/END:VCARD/END:VCARD

The boss's new name is now the agent's. Five corpus fixtures carry an embedded agent.

A replayed `END` truncates the card, because a bare record's `END:VCARD` diffs as an added property and is inserted before the wrapped card's real one, so the reparse stops at the first and every line after it is gone. The merged card is then not a serialization fixpoint.

## What

A `BEGIN` or `END` line is envelope rather than property, so `instances` skips both alongside the `VERSION` indicator it already skipped: neither is diffed, matched, removed or replayed. `finish` looks for the line an addition follows among the outer card's lines only, so an addition to the outer card never lands inside an embedded one, and falls back to the end of the card as before.

## Judgement calls, for review

**An embedded card's own lines are still diffed.** The obvious stronger move is to make the whole embedded run opaque, one value of the property carrying it, which is how the parser describes it. It was rejected: a right-side edit to a line the agent owns would then neither land nor be reported, which trades this change's silent loss for another one, and the merge's cardinal rule is that a change either lands or is named. Only the placement of an addition and the envelope lines themselves are kept out; `an_edit_inside_an_embedded_agent_still_merges` pins the difference.

**A stray `END` is envelope, not a property.** A bare record carrying `END:VCARD` gets that line skipped rather than merged, so a right-side copy adding one contributes nothing. Treating it as a property is what truncates the merged card, and no version gives `END` a meaning as one.
