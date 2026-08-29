---
cairn: log
change: merged-lines-are-terminated
landed: 2026-08-29
---

# A line that stops being last is terminated

A record read without a trailing newline leaves its last line with an empty ending, and the merge wrote additions after it without terminating it, so the added line landed inside the previous line's value and was destroyed, silently. The same gluing could put a bare record's unterminated line in front of a wrapped card's `END` and make the merged card fail to reparse at all.

Once the deferred removals and additions have run, every line of the merged card but its last carries a line ending. A card already terminated is untouched, and a line still last keeps its empty ending.

The byte-preservation law was narrowed to match: a line whose base spelling carries no ending may appear in the merged card with the default `\r\n` appended, and nothing else about it may change. The fuzz target no longer skips a card carrying an unterminated line.

Spec updated: `merge` (ADDED: the merged card is a card; the capability's own preamble now names that one exception to byte preservation).
