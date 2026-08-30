---
cairn: spec
capability: json-codecs
status: current
---

# JSON codecs

jCard (RFC 7095) and JSContact (RFC 9553, mapped by RFC 9555), both sitting on the decoded model behind their own cargo features.

### Requirement: The JSON boundary is a raw value, not serde derives

Both codecs SHALL cross the boundary as a raw `serde_json::Value` rather than through `serde` impls on any vcard type.

One type can have two JSON spellings (jCard versus JSContact), so serde, which keys one representation per type, is the wrong tool; a raw-value boundary also keeps the public API free of a serialization commitment. `serde_json` is pulled in its `no_std` alloc-only mode.

#### Scenario: No serde in the public API
- GIVEN the crate built with all features
- WHEN the public API is inspected
- THEN no vcard type implements `Serialize` or `Deserialize`

### Requirement: jCard export follows the RFC, import is liberal

`Vcard::to_jcard` SHALL lowercase names, move a group prefix to the `group` parameter, move `VALUE` to the type slot and re-spell dates extended. `Vcard::from_jcard` SHALL accept anything structurally sound, borrowing the JSON and resolving value kinds through the property specs.

The type slot goes back to a `VALUE` parameter only where the wire form needs it.

#### Scenario: A grouped property
- GIVEN a card carrying `item1.EMAIL`
- WHEN it is exported to jCard
- THEN the name is `email` and `item1` appears as the `group` parameter

### Requirement: JSContact conversion is lossless in both directions

`Vcard::to_jscontact` and `Vcard::from_jscontact` SHALL both be infallible aside from a non-object import root, preserving anything they cannot map first-class.

`TYPE`, `PREF` and `PROP-ID` map to contexts and features, `pref`, and the object key. An unmappable property is preserved in `vCardProps`, a leftover parameter in `vCardParams` (both in jCard syntax), and an unknown Card member becomes a `JSPROP` property. That escape hatch is why the `jscontact` feature requires `jcard`.

#### Scenario: An unmappable property
- GIVEN a card carrying a property JSContact has no member for
- WHEN it is converted to a Card and back
- THEN the property returns unchanged, having travelled through `vCardProps`

### Requirement: Object types are spelled as the RFC names them

Every `@type` the JSContact export emits SHALL be the object type name RFC 9553 registers, never a name from an earlier draft. The URI-valued resource collections SHALL therefore be tagged `Media`, `CryptoKey`, `Calendar`, `Link` and `Directory`.

Import stays liberal and ignores `@type` entirely, so a Card written with a draft-era name still converts back unchanged.

#### Scenario: A card carrying a URL
- GIVEN a card carrying `URL:https://example.org`
- WHEN it is converted to a JSContact Card
- THEN the entry under `links` is tagged `"@type": "Link"`

### Requirement: RFC 9554 is modeled first-class, not escaped

The RFC 9554 properties (`CREATED`, `GRAMGENDER`, `LANGUAGE`, `PRONOUNS`, `SOCIALPROFILE`), its parameters, and the RFC 9555 `JSPROP` property and `JSPTR` parameter SHALL each have a marker and spec, so the JSContact conversion maps them directly instead of dropping them into the escape hatch.

#### Scenario: A pronouns property
- GIVEN a 4.0 card carrying `PRONOUNS`
- WHEN it is converted to a JSContact Card
- THEN it maps to the corresponding member rather than into `vCardProps`
