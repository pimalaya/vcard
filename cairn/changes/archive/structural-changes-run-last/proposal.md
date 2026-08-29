---
cairn: change
id: structural-changes-run-last
status: landed
created: 2026-08-29
---

# The replay's ordering guarantee is written down and asserted

## Why

A sibling merge in ical-rs was found addressing properties by position and renumbering them: a removal shifted the lines that later actions still addressed by their old index, so an edit landed on the wrong property. The two merges share a lineage, so the shape is worth checking here.

It does not reproduce. This merge already separates the two phases: every value, component and parameter edit runs in place on the left clone while no line moves, removals are deferred and then applied on descending indices, and each addition recomputes where it goes against the card as it stands. Nothing addressed by index is read after an index has moved.

What was missing is that none of this was written down as a requirement, so nothing stops a future change from moving a removal into the replay loop, and no test named the invariant.

## What

The requirement covering the two phases says so, and `a_removal_does_not_renumber_what_follows_it` pins it: a right side that removes a line near the top of a card while editing three lines below it merges cleanly.

No behaviour changes.
