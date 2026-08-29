---
cairn: log
change: a-degenerate-record-is-out-of-fixpoint-scope
landed: 2026-08-29
---

# A bare record carrying an envelope line is out of the fixpoint law's scope

No behaviour changed. The merge fuzz target reported a merged card that was not a serialization fixpoint, on three inputs that were all fixpoints themselves. The left copy was a bare record whose second line was `BEGIN:vCard`, so the parser saw no envelope and kept every line as an ordinary property; removing the first property promoted that `BEGIN` to the front, and the same bytes then read as an enveloped card ending at the first `END`.

Nothing was invented, moved or corrupted: the document changed meaning because the envelope is positional. The fixpoint law now skips a merged card that is a bare record carrying an envelope line, and the merge capability records the scope beside the emptied-card case. An enveloped card holding an embedded agent, which is the legitimate way a card carries `BEGIN` and `END` among its lines, still runs the law in full.

Spec updated: `merge` (MODIFIED: the merged card is a card).
