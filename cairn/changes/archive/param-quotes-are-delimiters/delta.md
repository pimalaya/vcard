---
cairn: delta
change: param-quotes-are-delimiters
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A parameter value is encoded by RFC 6868, not by the text escapes

A parameter value SHALL be decoded and encoded by RFC 6868 section 3.1: `^n` reads as a newline, `^^` as a caret, `^'` as a double quote, and any other caret sequence, a trailing lone caret included, stays exactly as written, which section 3.1 requires rather than merely permits. A backslash SHALL be content in both directions, since RFC 6350 section 3.3 gives a parameter value no escapes at all and RFC 6868 section 3.2 forbids adding the backslash ones.

RFC 6868 updates RFC 6350 and no earlier specification, so the rules SHALL apply to vCard 4.0 alone. A 2.1 or 3.0 parameter carries its caret literally, and a parameter node SHALL therefore carry the escaping mode of the card it was parsed from, stamped once `VERSION` is known, as a value node already does. `VcardEscaper` SHALL name the three versions separately, 3.0 and 4.0 escaping a value identically but only 4.0 encoding a parameter.

The double quotes RFC 6350 section 3.3 wraps a `param-value` in SHALL be delimiters rather than content: decoding a parameter SHALL strip a balanced surrounding pair before resolving the carets, and encoding one SHALL wrap the encoded text in a pair when it carries a `,`, a `;` or a `:`, the delimiters a bare `SAFE-CHAR` run may not hold. A double quote cannot reach that test in 4.0, the caret encoding having already spelled it `^'`.

Quoting is a vCard 3.0 and 4.0 rule, RFC 2425 section 5.1 defining the `quoted-string` RFC 2426 inherits, so `VcardEscaper` SHALL answer for it separately from the caret encoding: a 2.1 parameter has no quoting and its double quote is content. A 3.0 value carrying both a double quote and a delimiter has no conformant spelling, the version having no way to encode the quote, and SHALL be written quoted rather than dropped.

An unbalanced quote SHALL be content, so a value the wire left open decodes as it stands rather than losing a delimiter it never closed.

#### Scenario: The three sequences
- GIVEN `LABEL=a^nb^^c^'d` in a 4.0 card
- WHEN it is decoded and encoded again
- THEN it reads `a`, a newline, `b^c"d`, and comes back as `LABEL=a^nb^^c^'d`

#### Scenario: A caret before an ordinary letter
- GIVEN `LABEL=a^xb^`
- WHEN it is decoded
- THEN it reads `a^xb^`, the caret and what follows staying as written

#### Scenario: A backslash in a parameter
- GIVEN `X-PATH=C:\temp\note.txt`
- WHEN it is decoded
- THEN the value keeps both separators rather than losing them to a text escape

#### Scenario: A caret in a 3.0 parameter
- GIVEN `ADR;LABEL=a^nb` in a 3.0 card
- WHEN it is decoded
- THEN it reads `a^nb`, the version predating RFC 6868

#### Scenario: A quoted parameter through a round trip
- GIVEN `GEO="geo:37.386,-122.083"`
- WHEN it is decoded
- THEN it reads `geo:37.386,-122.083`, and encoding it again puts the quotes back, the value carrying a `:` and a `,`

#### Scenario: A quoted value needing no quotes
- GIVEN `TYPE="work"`
- WHEN it is decoded and encoded again
- THEN it reads `work` and comes back as `TYPE=work`, the quotes having nothing to protect

#### Scenario: A double quote inside a 4.0 parameter
- GIVEN a decoded `LABEL` reading `say "hi", then go`
- WHEN it is encoded
- THEN it comes back as `LABEL="say ^'hi^', then go"`, the quote encoded and the pair added for the comma

#### Scenario: A quote a 2.1 card wrote
- GIVEN `X-FOO="bar"` in a 2.1 card
- WHEN it is decoded
- THEN it reads `"bar"`, the version having no quoting for the pair to delimit

#### Scenario: An unbalanced quote
- GIVEN `TYPE="work` in a 4.0 card
- WHEN it is decoded
- THEN it reads `"work`, the pair being unbalanced

## REMOVED Requirements
