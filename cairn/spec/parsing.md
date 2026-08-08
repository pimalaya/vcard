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

The parser SHALL preserve every byte of a parsed card that it does not normalise, so serializing an unedited card reproduces the input, and editing one property leaves every other byte intact.

Normalisation is the exhaustive list under "Line normalisation" below. Everything else, including line endings, parameter order, casing, whitespace inside a value and unknown vocabulary, survives byte for byte.

#### Scenario: An unedited card
- GIVEN a card whose lines are unfolded, with no blank lines and no QUOTED-PRINTABLE soft breaks
- WHEN it is parsed and serialized
- THEN the output is byte-identical to the input

#### Scenario: One edited property
- GIVEN a parsed card
- WHEN one property value is set through its lens cursor
- THEN only that value's bytes differ in the output

### Requirement: Serialization fixpoint

Serializing a parsed card SHALL produce bytes that reparse to the same bytes, whatever the input, so output is always stable under a second pass.

#### Scenario: A folded input
- GIVEN a card folded at 75 octets
- WHEN it is parsed, serialized, and the output reparsed and serialized again
- THEN the two outputs are byte-identical

### Requirement: Line normalisation

The parser SHALL resolve, and not restore, exactly these wire artifacts: line folding, QUOTED-PRINTABLE soft line breaks, blank lines between content lines, and a leading folding whitespace on a line with no line to continue.

A dangling QUOTED-PRINTABLE soft-break marker (a trailing `=` on a value whose line declares `ENCODING=QUOTED-PRINTABLE`) is stripped, however it got there, since leaving it in would re-trigger soft-break joining on reparse and swallow the next line.

#### Scenario: A dangling continuation
- GIVEN a line beginning with folding whitespace that follows a dropped blank line
- WHEN the card is parsed and serialized
- THEN the leading whitespace is stripped so the line stays its own line and the output reparses unchanged

### Requirement: Liberal input

The parser SHALL accept any byte input without panicking, and SHALL reject only a name or a parameter that is not valid UTF-8.

A property *value* is held as raw bytes, so a value in a foreign character set (a vCard 2.1 `CHARSET`) survives byte for byte. A name or parameter must be UTF-8, as every version's grammar guarantees. Because of that, `to_bytes` is the byte-faithful serializer, while `Display` and `to_string` are a convenience that is lossy only for a non-UTF-8 value.

#### Scenario: Arbitrary bytes
- GIVEN any byte sequence
- WHEN it is handed to the parser
- THEN the call returns a card or a parse error, and never panics

### Requirement: Envelope-free and multi-card input

`VcardCst::parse` SHALL read one card, accepting a bare RFC 2425 directory record with no `BEGIN` / `END` envelope, and `VcardCst::parse_many` SHALL iterate every card in a file lazily.

#### Scenario: A bare record
- GIVEN an RFC 2425 record with no `BEGIN:VCARD` line
- WHEN it is parsed
- THEN it yields a card whose envelope is absent, and it serializes back without one

### Requirement: Non-recursive AGENT

An embedded vCard in an `AGENT` property SHALL stay raw text, re-parsed only through an explicit opt-in helper, and that helper SHALL descend exactly one level.

Recursion on untrusted input is a denial-of-service risk, which is why nesting is opt-in and bounded rather than automatic.

#### Scenario: A nested AGENT
- GIVEN a 2.1 card whose `AGENT` value is itself an escaped card carrying its own `AGENT`
- WHEN `VcardCst::agent` is called
- THEN the outer `AGENT` is re-parsed and the inner one is left as raw text
