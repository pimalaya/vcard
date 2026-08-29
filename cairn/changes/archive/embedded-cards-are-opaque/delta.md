---
cairn: delta
change: embedded-cards-are-opaque
---

## ADDED Requirements

### Requirement: An envelope line is not a property

The merge SHALL treat a `BEGIN` or `END` line among a card's lines as envelope rather than as a property: neither is diffed, matched, removed nor replayed, alongside the `VERSION` indicator. Replaying an `END` would otherwise close the merged card early and drop everything after it.

An addition SHALL be placed after the last line of the outer card sharing its name, never after a line of a card embedded in an `AGENT`, or at the end of the card when there is none. The embedded card's own lines are still diffed, so an edit to one of them lands and a divergent one is reported.

#### Scenario: An addition beside an embedded agent
- GIVEN a 2.1 card whose `AGENT` embeds a card carrying its own `FN`, and a copy adding an `FN` to the outer card
- WHEN they are merged
- THEN the added `FN` sits on the outer card, not on the agent

#### Scenario: A bare record carrying an END
- GIVEN a wrapped base card and a bare copy whose last line is `END:VCARD`
- WHEN they are merged
- THEN the merged card carries one `END` and reparses to itself

## MODIFIED Requirements

## REMOVED Requirements
