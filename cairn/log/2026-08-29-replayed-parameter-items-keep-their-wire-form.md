---
cairn: log
change: replayed-parameter-items-keep-their-wire-form
landed: 2026-08-29
---

# A replayed parameter item is written as the right card spelled it

Found by the merge fuzz target: the merged card no longer parsed, failing with `MissingPropertyColon("ADR;TYPE=WORK,P Waters Edge")`.

A parameter value is read through the value unescaper and written back verbatim, since a parameter's wire form is quoted rather than backslash-escaped, and the item replay pushed the decoded item. A wire `TYPE=a\nb` therefore decoded to an item holding a real line break, which the replay wrote into the middle of the line's head: the line ended there and the rest folded onto what followed.

The replay now writes the leaf the right card holds, as the whole-parameter replay beside it already did. The same asymmetry still lets a decoded card serialize a broken parameter, which is the parameter codec's question and not the merge's.

Spec updated: `merge` (ADDED: a replayed parameter item keeps its wire form).
