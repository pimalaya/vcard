---
cairn: delta
change: writing-a-line-break-into-a-2-1-value
---

## ADDED Requirements

### Requirement: A written value never ends its own line

Serializing a value SHALL escape a line break in every version, so a value carrying one stays one content line and the card parses back.

vCard 2.1 defines no line-break escape, so its writer emits `\n` and its reader still resolves `\;` alone: the two halves are deliberately not inverses, since the only alternative spelling of a line break in 2.1 is a card that does not parse.

#### Scenario: A note holding a line break in a 2.1 card
- GIVEN a 2.1 card whose `NOTE` value is set to text carrying a line break
- WHEN the card is serialized and parsed again
- THEN it parses, and the value is one line

## MODIFIED Requirements

## REMOVED Requirements
