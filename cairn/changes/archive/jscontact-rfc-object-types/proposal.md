---
cairn: change
id: jscontact-rfc-object-types
status: landed
created: 2026-08-09
---

# JSContact resource objects carry their draft `@type`, not the RFC one

The JSContact export tags every URI-valued resource object with a type name from the pre-RFC JSContact drafts: `MediaResource` for PHOTO / LOGO / SOUND, `CryptoResource` for KEY, `CalendarResource` for CALURI / FBURL, `LinkResource` for URL and `DirectoryResource` for SOURCE. RFC 9553 §2.6 names these object types `Media`, `CryptoKey`, `Calendar`, `Link` and `Directory`.

A strict server rejects the draft spelling. Cardamum's 2026-08-09 Fastmail JMAP run found that a contact as ordinary as one carrying `URL:` cannot be written at all: `ContactCard/set` answers `InvalidProperties { properties: ["links/1/@type"] }`, and the same for `PHOTO` (`media/1/@type`) and `KEY` (`cryptoKeys/1/@type`). Both spellings were probed against the live server for all five collections: every RFC name is accepted, every draft name is rejected. The same projection is what cardamum-android writes with, so the blast radius is every Pimalaya product that speaks JMAP contacts.

## What changes

The five `@type` string literals passed to the `resource` helper in src/jscontact.rs, and the doc comments and test fixtures naming them.

Nothing changes on the import side: `Import::member` already ignores `@type` when reading a resource object (it is consumed before the parameter split), so a Card written by an older version still converts back property for property. Conversion stays lossless in both directions, which is the requirement this defect was hiding under.
