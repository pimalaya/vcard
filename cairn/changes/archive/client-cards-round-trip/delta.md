---
cairn: delta
change: client-cards-round-trip
---

## ADDED Requirements

### Requirement: A card is written as another version

`Vcard::to_version` SHALL rewrite a decoded card between vCard 3.0 (RFC 2426) and 4.0 (RFC 6350 Appendix A), keeping its groups (`item1.`) and every property and parameter no rule names. Up to 4.0, from any earlier version: a `pref` type is `PREF=1`; an inline `PHOTO`, `LOGO`, `SOUND` or `KEY` (`ENCODING=b` with its `TYPE` subtype) is a `data:` URI, and a referenced one drops `VALUE=uri`; an `AGENT` URI is `RELATED;TYPE=agent`, and an inline agent card, which 4.0 has no place for, is dropped; Apple's `X-ADDRESSBOOKSERVER-KIND` and `X-ADDRESSBOOKSERVER-MEMBER` are `KIND` and `MEMBER`. Down from 4.0 to 3.0, each rule runs the other way, a `PREF` other than 1 staying as written. Any other pair of versions SHALL change the version alone.

#### Scenario: An iOS group
- GIVEN a 3.0 card with `X-ADDRESSBOOKSERVER-KIND:group` and two `X-ADDRESSBOOKSERVER-MEMBER`
- WHEN it is written as 4.0 and back as 3.0
- THEN the 4.0 card states `KIND:group` and two `MEMBER`, and the 3.0 card is the original byte for byte

#### Scenario: An iOS contact
- GIVEN a 3.0 card with a `pref` type, an `item1.` group with `X-ABLabel`, an inline JPEG `PHOTO`, a referenced `LOGO` and an `AGENT` URI
- WHEN it is written as 4.0 and back as 3.0
- THEN the 4.0 card states `PREF=1`, the group, a `data:image/jpeg;base64,` `PHOTO`, a bare `LOGO` URI and `RELATED;TYPE=agent`, and the 3.0 card is the original byte for byte

### Requirement: A GENDER with no identity is written as its sex alone

Writing a decoded `GENDER` whose identity is empty SHALL write the sex component alone, so `GENDER:M` decodes and writes back as `GENDER:M`, not `GENDER:M;`.

### Requirement: Client cards round-trip through JSContact

A card as clients write it SHALL come back from its JSContact Card line for line, every `X-` property and parameter included, once normalized: line and parameter order, `TYPE` list shape and case, the case of a `VALUE`, a `VALUE` stating the default type, a date or timestamp spelled basic, a `\:` (no RFC 6350 escape) read as `:`, `CATEGORIES` order (keywords are a set), folds and name case. A 3.0 card goes up to 4.0 through `Vcard::to_version` first and back down after. Importing, a phone `number` with a `tel:` scheme SHALL be a `TEL;VALUE=uri`, so a URI-valued `TEL` keeps its value type.

#### Scenario: Cards as clients write them
- GIVEN each of tests/corpus/clients: an iOS contact (3.0, `item` groups with `X-ABLabel`, `X-ABRELATEDNAMES`, `X-ABDATE`, inline photo, `X-SOCIALPROFILE` with `X-` parameters), an iOS group (3.0), a Google contact (3.0, `BDAY:--0315`, `\:` in a URL, referenced photo), an Outlook contact (3.0, `X-MS-*`, `AGENT` URI, quoted `LABEL`) and a Thunderbird contact (4.0, `VALUE=URI` `TEL`, `data:` photo, an `X-` property with list-valued `X-` parameters)
- WHEN each is converted to a Card and back
- THEN it comes back line for line once normalized, and a second conversion of the Card equals the first

## MODIFIED Requirements

## REMOVED Requirements
