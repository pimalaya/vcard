---
cairn: delta
change: a-line-remembers-its-wire-shape
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Round-trip fidelity

The parser SHALL preserve every byte of a parsed card, its folds, its blank lines and its QUOTED-PRINTABLE soft breaks included, so serializing an unedited card reproduces the input exactly, and editing one property leaves every other byte intact.

#### Scenario: A folded real-world card
- GIVEN a card folded at 75 octets, with blank lines between its properties
- WHEN it is parsed and serialized
- THEN the output is byte-identical to the input

#### Scenario: One edited property
- GIVEN a parsed, folded card
- WHEN one property value is set through its lens cursor
- THEN every line but the edited one keeps its original wire shape

### Requirement: Line normalisation

The parser SHALL resolve the wire shape of a line into logical content for every layer above it, and SHALL restore that shape on output: folded continuation lines, QUOTED-PRINTABLE soft line breaks, blank lines before a content line, and a leading folding whitespace on a line with no line to continue. A missing final line break stays absent on output.

An edited value SHALL drop the recorded shape of its own line rather than re-apply fold points that no longer match its bytes.

A dangling QUOTED-PRINTABLE soft-break marker (a trailing `=` on a value whose line declares `ENCODING=QUOTED-PRINTABLE`) SHALL leave the logical value, since leaving it in would re-trigger soft-break joining on reparse and swallow the next line, and SHALL be restored from the wire shape.

The leading whitespace SHALL be stripped from the assembled logical line, not only from its first physical line, so a whitespace-only line whose continuation carries more than the one fold marker does not leave the leftover in front of the name.

#### Scenario: A folded line
- GIVEN `NOTE:foo\r\n bar\r\n`
- WHEN it is parsed
- THEN the line holds the logical value `foobar` and serializes folded exactly as it arrived

#### Scenario: A continuation of a whitespace-only line
- GIVEN the input `"   \r\n  A:b\r\n"`
- WHEN it is parsed
- THEN the line is named `A`, not `" A"`, and it serializes back to the input

### Requirement: Envelope-free and multi-card input

`VcardCst::parse` SHALL read one card, accepting a bare RFC 2425 directory record with no `BEGIN` / `END` envelope, and `VcardCst::parse_many` SHALL iterate every card in a file lazily.

Blank lines between cards SHALL belong to the card that follows them, and blank lines after the last card to that card, so concatenating what `parse_many` yields reproduces the file byte for byte.

#### Scenario: A bare record
- GIVEN an RFC 2425 record with no `BEGIN:VCARD` line
- WHEN it is parsed
- THEN it yields a card whose envelope is absent, and it serializes back without one

#### Scenario: A file ending in a blank line
- GIVEN a multi-card file with blank lines between the cards and one after the last
- WHEN every card is parsed and their bytes concatenated
- THEN the result is byte-identical to the file

## REMOVED Requirements
