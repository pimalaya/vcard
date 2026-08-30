---
cairn: delta
change: rfc6868-param-encoding
---

## ADDED Requirements

### Requirement: A parameter value is encoded by RFC 6868, not by the text escapes

A parameter value SHALL be decoded and encoded by RFC 6868 section 3.1: `^n` reads as a newline, `^^` as a caret, `^'` as a double quote, and any other caret sequence, a trailing lone caret included, stays exactly as written, which section 3.1 requires rather than merely permits. A backslash SHALL be content in both directions, since RFC 6350 section 3.3 gives a parameter value no escapes at all and RFC 6868 section 3.2 forbids adding the backslash ones.

RFC 6868 updates RFC 6350 and no earlier specification, so the rules SHALL apply to vCard 4.0 alone. A 2.1 or 3.0 parameter carries its caret literally, and a parameter node SHALL therefore carry the escaping mode of the card it was parsed from, stamped once `VERSION` is known, as a value node already does. `VcardEscaper` SHALL name the three versions separately, 3.0 and 4.0 escaping a value identically but only 4.0 encoding a parameter.

A value the wire spelled inside its own double quotes SHALL keep that pair on the way out, only what they enclose being encoded. The decoded model holds a parameter exactly as it was written, delimiters included, so encoding the surrounding pair would strip the quoting off every quoted URI.

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
- WHEN it is decoded and encoded again
- THEN the bytes are the ones it arrived as

## MODIFIED Requirements

### Requirement: Values are compared on the raw node

The merge SHALL decide whether two copies hold the same value by comparing their raw value nodes component by component, over every component of the value, never through the decoded projection, which reads only a value's first `;`-component. An identity read off a value SHALL be read from the same raw node, for the same reason.

Two components agree when they decode to the same list of items, so a difference in escaping is a difference. An absent component and an all-empty one agree, so a trailing empty component is not a change.

Two parameters SHALL be compared the same way, on their raw nodes and value by value: a single-valued parameter decodes its first value alone, so two parameters differing past their first `,` decode alike and the edit is never reported. Where the two nodes carry different escaping modes they share no decoding to compare through, and only identical bytes are then certainly the same parameter. The replay SHALL address the right card's parameter node by name and ordinal, a decoded parameter not being a key either.

#### Scenario: A photo payload past the first semicolon
- GIVEN a base card carrying `PHOTO:data:image/png;base64,AAAA` and a copy carrying a different payload
- WHEN they are merged
- THEN the change is reported and lands, as one photo leaving and another arriving

#### Scenario: A title past the first semicolon
- GIVEN a base `TITLE:boss;of;nothing` and two copies rewriting what follows the second `;`
- WHEN they are merged
- THEN the divergence is reported

#### Scenario: A parameter past its first comma
- GIVEN a base `ADR;LABEL=Ada,Lovelace` and a copy holding `ADR;LABEL=Ada,Byron`
- WHEN they are merged
- THEN the change is reported and the merged card carries it

### Requirement: A replayed parameter item keeps its wire form

An item the merge replays into a list parameter SHALL be written as the right card spelled it, never as its decoded text: a decoded item holds a real line break where the wire holds the RFC 6868 `^n`, so writing it back decoded would end the line in the middle of its head.

#### Scenario: A type value holding an encoded line break
- GIVEN a base `TEL;TYPE=work` and a right copy adding the item `a\nb^nc`
- WHEN they are merged
- THEN the merged line reads `TEL;TYPE=work,a\nb^nc` and parses back to itself
