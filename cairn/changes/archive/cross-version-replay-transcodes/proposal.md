---
cairn: change
id: cross-version-replay-transcodes
status: landed
created: 2026-08-29
---

# A value replayed across versions is transcoded, not copied

## Why

The merged card keeps the left card's `VERSION`, and a version change is not reconciled. The replay copies the right side's changes as that card's raw value bytes, so a value crosses from one escaping mode into the other unchanged and its meaning changes with it:

    base   VERSION:2.1  NOTE:a,b     the text `a,b`
    right  VERSION:4.0  NOTE:a\,c    the text `a,c`

    merged VERSION:2.1  NOTE:a\,c    the text `a\,c`

vCard 2.1 has no comma escape in a value, so the backslash the 4.0 card wrote to escape its comma becomes a literal backslash in the merged card. The module documents the version rule and stops there, so a caller reasonably expects the merge to refuse or to transcode rather than to corrupt.

## What

A value node replayed from the right card is re-encoded for the merged card's escaping mode when the two differ: each component is decoded with the rules the right card was written under and written back with the rules the merged card reads. That covers a whole-value change and a whole line the right side added or restored. The finer replays already went through the decoded model, so they transcoded already, and a parameter value is never escaped.

A node already written for the merged card's mode is cloned unchanged, so every merge of one version's cards is untouched and stays byte for byte what it was.

Two cards of different versions also share no decoding to compare values through, so across versions the merge decides whether two values are the same on their raw bytes: `http\://x` reads as itself in 2.1 and as `http://x` later, and comparing through the decoded value would call one line at two versions a change and rewrite it. The fuzz target found that as soon as the replay started transcoding what it used to copy.

## Judgement call, for review

**The merge transcodes rather than refusing.** `merge` is infallible by construction: it returns a report, not a `Result`, and a caller with three copies at two versions has no other reconciliation to fall back on. Refusing would mean a new failure mode in an API whose whole point is that it always produces a card, and a documented precondition nobody can check cheaply. Transcoding is lossy in the direction the versions are lossy (a 4.0 value carrying an escaped comma arrives in a 2.1 card as a literal one, which is what 2.1 spells it as), and it only rewrites the bytes of a line the right side changed anyway.
