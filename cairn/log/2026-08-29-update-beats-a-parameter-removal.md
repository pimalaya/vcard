---
cairn: log
change: update-beats-a-parameter-removal
landed: 2026-08-29
---

# An update beats a removal at parameter granularity too

The rule that an update beats a removal held for a whole property and nowhere else, so at parameter granularity the left action simply won and the outcome depended on which copy the caller passed as `left`: `merge(base, updated, removed)` kept `PREF=2` while `merge(base, removed, updated)` dropped it. Nothing was silent, but a caller that trusted the documented rule resolved conflicts wrongly.

The parameter arms of the replay grew the exception. A right-side update over a left-side removal is reported and lands, appended to the merged line since the removal took its position with it, and a right item edit whose parameter the left side removed restores that whole parameter from the right card instead of being dropped.

Restoring on the first such item edit also made the conflict count symmetric, since the following item edits then have nothing to collide against, so the swapped-sides law was strengthened from the collided fields to the number of conflicts.

Spec updated: `merge` (MODIFIED: conflicts are reported, never silently resolved away; the merge obeys its algebraic laws).
