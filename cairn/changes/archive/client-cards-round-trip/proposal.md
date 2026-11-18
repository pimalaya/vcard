---
cairn: change
id: client-cards-round-trip
status: landed
created: 2026-11-18
---

# Client cards round-trip

## Why

Pimalaya Server stores contacts as vCard (CardDAV) and serves them as JSContact (JMAP), and iOS writes vCard 3.0 while JSContact maps 4.0. The crate had no version conversion, and nothing checked a card as real clients write it coming back from JSContact line for line.

## What

`Vcard::to_version` between 3.0 and 4.0 (Apple groups, `pref`, inline media, `AGENT`), realistic client cards under tests/corpus/clients held to a line-for-line round trip through JSContact, and the two losses that check found fixed: `GENDER:M` written back as `GENDER:M;`, and a `TEL;VALUE=uri` coming back as text.
