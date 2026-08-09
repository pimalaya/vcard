---
cairn: log
change: jscontact-rfc-object-types
landed: 2026-08-09
---

# JSContact resource objects carry their RFC `@type`

The JSContact export tagged every URI-valued resource object with a pre-RFC draft name: `MediaResource` for PHOTO / LOGO / SOUND, `CryptoResource` for KEY, `CalendarResource` for CALURI / FBURL, `LinkResource` for URL, `DirectoryResource` for SOURCE. RFC 9553 §2.6 registers them as `Media`, `CryptoKey`, `Calendar`, `Link` and `Directory`, and a strict server enforces that: cardamum's 2026-08-09 Fastmail JMAP run could not write a contact carrying so much as a `URL`, the server answering `InvalidProperties { properties: ["links/1/@type"] }`. Both spellings were probed live against Fastmail for all five collections before the rename: every RFC name is accepted, every draft name rejected.

Export now emits the RFC names, and the capability [json-codecs](../spec/json-codecs.md) gained the requirement that it always will. Import is untouched: it already ignored `@type` on these entries, so a Card written by an older version still converts back property for property, and a test pins that. Another test pins the exported `@type` of every resource collection, including `SchedulingAddress`, which the drafts and the RFC spell the same way.

The projection is shared with cardamum-android, so the fix lands for both products at once.
