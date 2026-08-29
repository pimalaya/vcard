---
cairn: log
change: embedded-cards-are-opaque
landed: 2026-08-29
---

# An embedded card owns its own lines

A vCard 2.1 `AGENT` embeds a whole card that the parser keeps verbatim among the outer card's properties, and the merge skipped only `VERSION`, so those lines were diffed, matched and moved like any other. An `FN` added to the outer card landed inside the agent, silently, and a bare record's `END:VCARD` diffed as an added property, was inserted before the real one, and truncated the merged card at the reparse.

A `BEGIN` or `END` line is now envelope rather than property, skipped alongside `VERSION`, and an addition follows the last line of the *outer* card sharing its name. The embedded card's own lines are still diffed, so an edit inside an agent still lands: making the whole run opaque would have dropped it unreported, trading one silent loss for another.

The fuzz target no longer skips a card carrying an embedded envelope.

Spec updated: `merge` (ADDED: an envelope line is not a property).
