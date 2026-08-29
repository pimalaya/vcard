---
cairn: log
change: matching-prefers-identical-bytes
landed: 2026-08-29
---

# Among interchangeable instances, the identical bytes pair first

Found by the merge fuzz target: a line all three copies carried byte for byte was gone from the merged card. A card carrying one property three times, one of them ending `\r\n` and two ending `\n`, decodes to three interchangeable instances, so the equality pass matched them in source order and removed a line whose exact bytes every copy carried, keeping one whose bytes only one of them did.

The matching now runs a pass on identical serialized bytes between the `PID` pass and the decoded-equality pass. Byte equality implies decoded equality, so it only refines which of several equal candidates is chosen: the pairing that rewrites nothing comes first.

Spec updated: `merge` (MODIFIED: instance matching is PID, then equality, then position).
