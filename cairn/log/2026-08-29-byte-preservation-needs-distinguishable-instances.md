---
cairn: log
change: byte-preservation-needs-distinguishable-instances
landed: 2026-08-29
---

# Byte preservation is stated for a card whose instances can be told apart

No behaviour changed. The fuzz target's byte-preservation law now skips a card carrying two instances of one property with the same content, differing at most in their line ending, and the merge capability records the scope.

Three matchings run against the base, one per side, and each is free to pair interchangeable copies differently. Every such pairing preserves the same content and none is more right than another, but they disagree about which copy a removal takes and which spelling survives. Two real defects surfaced on the way to that conclusion and were repaired rather than papered over: two instances under one `PID` now pair by equality first, and among interchangeable instances the identical bytes pair first.

The content of such a card stays covered by the completeness law, the differential and the identity laws, so a duplicate that is lost rather than respelled still fails the suite.

Spec updated: `merge` (ADDED: byte preservation needs distinguishable instances).
