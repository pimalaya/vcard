---
cairn: change
id: closed-value-content
status: landed
created: 2026-08-31
---

# Delta

## ADDED Requirements

### Requirement: A closed value vocabulary is validated
Where a definition closes the content of a value, `validate` SHALL check it and report what it found. A property declares that through `VcardPropSpec::invalid_value`, which defaults to allowing everything: the rest of the spec describes a property's shape, and this member alone describes what may be inside its value.

`GENDER`'s sex component SHALL be one of `M`, `F`, `O`, `N`, `U` or empty (RFC 6350 6.2.7), `PROFILE`'s value SHALL be `VCARD` (RFC 2426 3.6.3), and `CLIENTPIDMAP`'s first field SHALL be a positive integer (RFC 6350 6.7.7). Matching SHALL be case-insensitive, RFC 5234 making a quoted ABNF literal so.

A vocabulary whose grammar ends in `iana-token / x-name` is open and SHALL NOT be checked. `KIND`, `CLASS`, `GRAMGENDER`, `CALSCALE`, `PHONETIC` and every `TYPE` set are open in exactly that way, and rejecting a value outside their listed ones would refuse cards that conform.

### Requirement: A constrained parameter value is validated
`PREF` SHALL be an integer from 1 to 100 (RFC 6350 5.3), `PID` one or more digits optionally followed by a dot and more digits (RFC 6350 5.5), and `DERIVED` either `true` or `false` (RFC 9554 3.4). Those constraints do not vary by the property carrying the parameter, so one check SHALL serve every appearance rather than each property restating it.

### Requirement: Content validation is not format validation
What `validate` checks inside a value SHALL be closed vocabularies and small integers. Dates, URIs, UTC offsets and language tags have grammars too, and checking them is a different undertaking with a far fuzzier edge: the crate would carry a URI parser to answer a question no caller asked.

Reading SHALL stay maximally liberal either way. A card carrying a value outside its vocabulary SHALL still parse and still round-trip byte for byte, and the decoded value SHALL stay the open type it is rather than becoming an enum. Strictness lives in `validate`, which is where a caller goes to ask.

## MODIFIED Requirements

## REMOVED Requirements
