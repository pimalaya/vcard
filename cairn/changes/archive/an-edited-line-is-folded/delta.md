---
cairn: delta
change: an-edited-line-is-folded
---

## ADDED Requirements

### Requirement: An edited or built line is folded

A 3.0 or 4.0 content line with no recorded shape, one built or encoded from the model, SHALL be folded when longer than 75 octets (RFC 6350 3.2): a line break and a single space before every continuation, each physical line 75 octets at most, its line ending excluded and the continuation space included. A fold SHALL NOT land inside a UTF-8 sequence. The break SHALL be the line's own ending, `\r\n` unless the line ends on a bare `\n`. A decoded card's text output SHALL fold its lines as its encoding does.

A line a cursor edit changed SHALL keep every recorded piece before the first byte the edit changed, and SHALL be folded from the last kept fold onward with that fold's whitespace and break, under the same rules. An edit keeping the line's length is no exception: its old folds do not carry over past the changed byte.

A line SHALL NOT be refolded when it is a vCard 2.1 line, since 2.1 unfolds as RFC 822 does and keeps the whitespace, so an added fold would change its value for a 2.1 reader, or when it declares `QUOTED-PRINTABLE`, since a fold after one of its `=` would read back as a soft break. Such a line goes out with the pieces kept before its edit and unfolded after them. A line pushed raw keeps the shape it was written in, and a parsed line nothing edited keeps its own.

#### Scenario: A long note edited at its end
- GIVEN a parsed 4.0 `NOTE` of 300 octets folded every 74 octets with a tab
- WHEN its last word is replaced by a longer one
- THEN every physical line before the one holding the edit comes out byte for byte, the rest is folded with a tab, no physical line exceeds 75 octets, and the output reparses to the edited value

#### Scenario: An edit moving a two-octet character across an old fold
- GIVEN a parsed 4.0 `NOTE` folded right after an `é`
- WHEN it is set to a value of the same length whose `é` sits one octet further
- THEN no physical line ends inside the `é`, and the output reparses to the edited value

#### Scenario: A 2.1 line
- GIVEN a parsed 2.1 `NOTE` and a property pushed into the same card, both longer than 75 octets
- WHEN the note is edited and the card serialized
- THEN neither line is folded

## MODIFIED Requirements

### Requirement: Line normalisation

The parser SHALL resolve the wire shape of a line into logical content for every layer above it, and SHALL restore that shape on output: folded continuation lines, QUOTED-PRINTABLE soft line breaks, blank lines before a content line, and a leading folding whitespace on a line with no line to continue. A missing final line break stays absent on output.

An edited value SHALL NOT re-apply fold points that no longer match its bytes: a cursor edit keeps those before the first changed byte and is folded after them, and a shape whose recorded length no longer matches is dropped (see the requirement above).

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

## REMOVED Requirements
