---
cairn: delta
change: quoted-parameter-values
---

## ADDED Requirements

### Requirement: A quoted parameter value is opaque

The line splitter SHALL treat a double-quoted parameter value as opaque, per RFC 6350 section 3.3: neither the `:` separating the head from the value nor the `;` separating one parameter from the next is recognised inside one.

A head carrying an unbalanced quote SHALL still parse: with no `:` outside quotes the splitter falls back to the first `:` anywhere, so a malformed line yields a line rather than an error.

#### Scenario: The RFC 6350 section 6.3.1 address
- GIVEN a line reading `ADR;GEO="geo:12.3457,78.910";TYPE=work:;;123 Main Street;...`
- WHEN it is parsed
- THEN it carries two parameters, `GEO` holding the whole quoted URI, and the address components are not shifted

## MODIFIED Requirements

## REMOVED Requirements
