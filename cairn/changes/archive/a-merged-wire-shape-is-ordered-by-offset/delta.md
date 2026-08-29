---
cairn: delta
change: a-merged-wire-shape-is-ordered-by-offset
---

## MODIFIED Requirements
### Requirement: Line normalisation

The parser SHALL resolve the wire shape of a line into logical content for every layer above it, and SHALL restore that shape on output: folded continuation lines, QUOTED-PRINTABLE soft line breaks, blank lines before a content line, and a leading folding whitespace on a line with no line to continue. A missing final line break stays absent on output.

An edited value SHALL drop the recorded shape of its own line rather than re-apply fold points that no longer match its bytes.

A dangling QUOTED-PRINTABLE soft-break marker (a trailing `=` on a value whose line declares `ENCODING=QUOTED-PRINTABLE`) SHALL leave the logical value, since leaving it in would re-trigger soft-break joining on reparse and swallow the next line, and SHALL be restored from the wire shape.

A recorded shape SHALL go back out in offset order, whichever of the tokeniser and the line splitter recorded each piece, and two pieces recorded at one offset SHALL keep the order they were recorded in. A value ending on two `=` is recorded by both at once, the soft break past the last logical byte and the dangling `=` before it, and emitting them in list order writes a line break into the middle of the value.

The leading whitespace SHALL be stripped from the assembled logical line, not only from its first physical line, so a whitespace-only line whose continuation carries more than the one fold marker does not leave the leftover in front of the name.

#### Scenario: A folded line
- GIVEN `NOTE:foo\r\n bar\r\n`
- WHEN it is parsed
- THEN the line holds the logical value `foobar` and serializes folded exactly as it arrived

#### Scenario: A continuation of a whitespace-only line
- GIVEN the input `"   \r\n  A:b\r\n"`
- WHEN it is parsed
- THEN the line is named `A`, not `" A"`, and it serializes back to the input

#### Scenario: A value ending on two soft-break markers
- GIVEN a `QUOTED-PRINTABLE` line whose value ends `x==`
- WHEN it is parsed and serialized
- THEN the output is the input, and it reparses to the same bytes rather than swallowing the line after it
