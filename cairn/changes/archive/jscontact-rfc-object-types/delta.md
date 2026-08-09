---
cairn: change
id: jscontact-rfc-object-types
status: landed
created: 2026-08-09
---

# Delta

## ADDED Requirements

### Requirement: Object types are spelled as the RFC names them

Every `@type` the JSContact export emits SHALL be the object type name RFC 9553 registers, never a name from an earlier draft. The URI-valued resource collections SHALL therefore be tagged `Media`, `CryptoKey`, `Calendar`, `Link` and `Directory`.

Import stays liberal and ignores `@type` entirely, so a Card written with a draft-era name still converts back unchanged.

#### Scenario: A card carrying a URL
- GIVEN a card carrying `URL:https://example.org`
- WHEN it is converted to a JSContact Card
- THEN the entry under `links` is tagged `"@type": "Link"`

## MODIFIED Requirements

## REMOVED Requirements
