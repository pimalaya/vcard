---
cairn: log
change: writing-a-line-break-into-a-2-1-value
landed: 2026-08-29
---

# A line break written into a vCard 2.1 value destroys the card

Found by the merge fuzz target after the merge repairs landed. The 2.1 escaper escaped `;` and nothing else, so a value carrying a line break was written raw, the break ended the content line, and the rest of the value became a line with no colon: the card no longer parsed. Reachable from the plain edit layer on any 2.1 card, and newly reachable from the merge, which transcodes a value replayed from a later version and so hands the 2.1 writer the line break the modern reader resolved.

The 2.1 escaper now writes a line break as `\n`. Its reader still resolves `\;` alone, so the two halves are deliberately not inverses: the reading is what every existing 2.1 card decodes through, and the only other spelling a writer has is a card that does not parse.

Spec updated: `parsing` (ADDED: a written value never ends its own line).
