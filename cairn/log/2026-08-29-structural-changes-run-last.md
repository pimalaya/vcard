---
cairn: log
change: structural-changes-run-last
landed: 2026-08-29
---

# The replay's ordering guarantee is written down and asserted

No behaviour changed. A sibling merge in ical-rs was found renumbering properties, a removal shifting the lines that later actions still addressed by their old index. The shape does not reproduce here: this merge already runs every in-place edit while no line moves, then applies deferred removals on descending indices, then places each addition against the card as it then stands.

That ordering is now a requirement rather than an accident of the code, and `a_removal_does_not_renumber_what_follows_it` pins it.

Spec updated: `merge` (MODIFIED: per-side action lists against a common base).
