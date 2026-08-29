---
cairn: spec
capability: parsing
status: current
---

# Parsing

The byte-faithful side of the crate, gated behind the `parser` feature. `VcardCst` models a card as an optional `BEGIN` / `END` envelope wrapping an ordered body of content lines, in source order. It knows nothing of what a property means: it reproduces the wire.

Parsing is maximally liberal, per Postel's law. Any real card is accepted, including properties, parameters and value types no version defines. Strictness lives only on the way out (see [conformance](./conformance.md)).

The card version is a decoded indicator, never a type parameter and never a separate dialect. The syntax tree ignores it; only the codec and the per-property spec branch on it, and only where escaping or a value's shape genuinely differ.

### Requirement: Round-trip fidelity

The parser SHALL preserve every byte of a parsed card, its folds, its blank lines and its QUOTED-PRINTABLE soft breaks included, so serializing an unedited card reproduces the input exactly, and editing one property leaves every other byte intact.

Line endings, parameter order, casing, whitespace inside a value and unknown vocabulary all survive byte for byte.

#### Scenario: A folded real-world card
- GIVEN a card folded at 75 octets, with blank lines between its properties
- WHEN it is parsed and serialized
- THEN the output is byte-identical to the input

#### Scenario: One edited property
- GIVEN a parsed, folded card
- WHEN one property value is set through its lens cursor
- THEN every line but the edited one keeps its original wire shape

### Requirement: Serialization fixpoint

Serializing a parsed card SHALL produce bytes that reparse to the same bytes, whatever the input, so output is always stable under a second pass.

#### Scenario: A folded input
- GIVEN a card folded at 75 octets
- WHEN it is parsed, serialized, and the output reparsed and serialized again
- THEN the two outputs are byte-identical

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

### Requirement: Liberal input

The parser SHALL accept any byte input without panicking, and SHALL reject only a name or a parameter that is not valid UTF-8.

A property *value* is held as raw bytes, so a value in a foreign character set (a vCard 2.1 `CHARSET`) survives byte for byte. A name or parameter must be UTF-8, as every version's grammar guarantees. Because of that, `to_bytes` is the byte-faithful serializer, while `Display` and `to_string` are a convenience that is lossy only for a non-UTF-8 value.

#### Scenario: Arbitrary bytes
- GIVEN any byte sequence
- WHEN it is handed to the parser
- THEN the call returns a card or a parse error, and never panics

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

### Requirement: Non-recursive AGENT

An embedded vCard in an `AGENT` property SHALL stay raw text, re-parsed only through an explicit opt-in helper, and that helper SHALL descend exactly one level.

Recursion on untrusted input is a denial-of-service risk, which is why nesting is opt-in and bounded rather than automatic.

#### Scenario: A nested AGENT
- GIVEN a 2.1 card whose `AGENT` value is itself an escaped card carrying its own `AGENT`
- WHEN `VcardCst::agent` is called
- THEN the outer `AGENT` is re-parsed and the inner one is left as raw text

### Requirement: A quoted parameter value is opaque

The line splitter SHALL treat a double-quoted parameter value as opaque, per RFC 6350 section 3.3: neither the `:` separating the head from the value nor the `;` separating one parameter from the next is recognised inside one.

A head carrying an unbalanced quote SHALL still parse: with no `:` outside quotes the splitter falls back to the first `:` anywhere, so a malformed line yields a line rather than an error.

#### Scenario: The RFC 6350 section 6.3.1 address
- GIVEN a line reading `ADR;GEO="geo:12.3457,78.910";TYPE=work:;;123 Main Street;...`
- WHEN it is parsed
- THEN it carries two parameters, `GEO` holding the whole quoted URI, and the address components are not shifted

### Requirement: A written value never ends its own line

Serializing a value SHALL escape a line break in every version, so a value carrying one stays one content line and the card parses back.

vCard 2.1 defines no line-break escape, so its writer emits `\n` and its reader still resolves `\;` alone: the two halves are deliberately not inverses, since the only alternative spelling of a line break in 2.1 is a card that does not parse.

#### Scenario: A note holding a line break in a 2.1 card
- GIVEN a 2.1 card whose `NOTE` value is set to text carrying a line break
- WHEN the card is serialized and parsed again
- THEN it parses, and the value is one line
