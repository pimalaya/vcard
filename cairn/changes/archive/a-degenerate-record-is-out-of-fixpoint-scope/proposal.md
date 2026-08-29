---
cairn: change
id: a-degenerate-record-is-out-of-fixpoint-scope
status: landed
created: 2026-08-29
---

# A bare record carrying an envelope line is out of the fixpoint law's scope

## Why

Found by the merge fuzz target: the merged card was not a serialization fixpoint.

The three inputs were all fixpoints and the merge did exactly what it was asked, removing one property. The left copy was a bare record whose *second* line was `BEGIN:vCard`, so the parser saw no envelope and kept every line, `BEGIN` and `END` included, as ordinary properties. Removing the first property promoted that `BEGIN` to the front, and the same bytes then read as an enveloped card that ends at the first `END`, dropping everything after it.

No line was invented, moved or corrupted. The document simply changed meaning because the format's envelope is positional, and no merge can preserve both the content it was given and a reading that depends on which line comes first.

## What

The fixpoint law skips a merged card that is a bare record carrying a `BEGIN` or `END` line among its properties, and says why at the site. The merge capability records the scope, next to the emptied-card case it already carries.

Every other shape still runs, including an enveloped card holding an embedded agent, which is the legitimate way a card carries `BEGIN` and `END` among its lines.

## Judgement call, for review

**The scope is drawn at the bare record, not at the envelope line.** A card with an envelope that also carries an embedded `BEGIN`..`END` is a real vCard 2.1 `AGENT` and keeps the law in full. Only a record with no envelope of its own and an envelope line inside it is exempt, which no real producer emits.
