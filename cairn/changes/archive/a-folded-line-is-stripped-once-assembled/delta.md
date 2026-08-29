---
cairn: delta
change: a-folded-line-is-stripped-once-assembled
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Line normalisation

The parser SHALL resolve, and not restore, exactly these wire artifacts: line folding, QUOTED-PRINTABLE soft line breaks, blank lines between content lines, and a leading folding whitespace on a line with no line to continue.

A dangling QUOTED-PRINTABLE soft-break marker (a trailing `=` on a value whose line declares `ENCODING=QUOTED-PRINTABLE`) is stripped, however it got there, since leaving it in would re-trigger soft-break joining on reparse and swallow the next line.

The leading whitespace is stripped from the assembled logical line, not only from its first physical line, so a whitespace-only line whose continuation carries more than the one fold marker does not leave the leftover in front of the name.

#### Scenario: A dangling continuation
- GIVEN a line beginning with folding whitespace that follows a dropped blank line
- WHEN the card is parsed and serialized
- THEN the leading whitespace is stripped so the line stays its own line and the output reparses unchanged

#### Scenario: A continuation of a whitespace-only line
- GIVEN the input `"   \r\n  A:b\r\n"`
- WHEN it is parsed
- THEN the line is named `A`, not `" A"`, and the output reparses unchanged

## REMOVED Requirements
