---
cairn: log
change: an-emptied-card-is-not-a-card
landed: 2026-08-29
---

# A merge that removes every line yields no card, and the law says so

Found by the merge fuzz target: the merged card failed to reparse with `MissingCrlf("")`, the parser's answer to no input at all. A bare RFC 2425 record carrying only envelope lines has no properties, so merging it against a record of one property removes that property and leaves zero bytes.

No behaviour changed. The reparse law, in the suite and in the fuzz target alike, no longer applies to a merged card with no bytes, and the merge capability says why: an empty document is not a card, and inventing a line to keep the output parseable would resurrect content nobody wrote. `removing_every_line_leaves_no_card` pins the outcome.

Spec updated: `merge` (MODIFIED: the merged card is a card).
