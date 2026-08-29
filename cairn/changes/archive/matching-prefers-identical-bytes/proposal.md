---
cairn: change
id: matching-prefers-identical-bytes
status: landed
created: 2026-08-29
---

# Among interchangeable instances, the identical bytes pair first

## Why

Found by the merge fuzz target: a line all three copies carried byte for byte was gone from the merged card.

A card may carry the same property twice, spelled two ways. The equality pass matches on the decoded property, so three `GENDER:M` lines, one of them ending `\r\n` and two ending `\n`, are all interchangeable to it. Pairing them in source order against a copy holding only the two `\n` ones matched the `\r\n` line first and left one `\n` line unmatched, so the merge removed a line whose exact bytes all three copies carried and kept one whose bytes only one of them did.

Nothing is lost semantically, since the instances are interchangeable, but the merge rewrote bytes it had no reason to touch, which is the property the byte-preserving edit layer exists to give.

## What

The matching gains a pass on identical serialized bytes, between the `PID` pass and the decoded-equality pass. Byte equality implies decoded equality, so it only refines which of several equal candidates is chosen: the pairing that changes nothing at all comes first.

The documented order becomes: `PID` and equality, then `PID`, then identical bytes, then equality, then position.
