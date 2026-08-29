---
cairn: log
change: cross-version-replay-transcodes
landed: 2026-08-29
---

# A value replayed across versions is transcoded, not copied

The merged card keeps the left card's `VERSION`, and the replay copied the right side's raw value bytes, so a value crossed from one escaping mode into the other unchanged: a 4.0 `NOTE:a\,c` arrived in a 2.1 card as the literal `a\,c`, since 2.1 has no comma escape.

A value node replayed from the right card is now re-encoded for the merged card's escaping mode when the two differ, which covers a whole-value change and a whole line the right side added or restored; the finer replays already went through the decoded model. A node already written for the merged card's mode is cloned unchanged, so a merge of one version's cards keeps its bytes.

Across versions the merge also decides whether two values are the same on their raw bytes rather than through a decoding the two cards do not share, so one line at two versions is not a change. The fuzz target found that half, once the replay started transcoding what it used to copy: a `URL:http\://www.other.com` all three copies carried was read as two different values and rewritten.

Spec updated: `merge` (ADDED: a replayed value carries its meaning, not its escaping).
