---
cairn: tasks
change: client-cards-round-trip
---

- [x] `Vcard::to_version` in src/vcard.rs
- [x] `VcardGender::emit` writes no empty identity component (src/tree/value/gender.rs)
- [x] A `tel:` phone number imports as `TEL;VALUE=uri` (src/jscontact/import.rs)
- [x] Fixtures tests/corpus/clients: ios-contact.vcf, ios-group.vcf, google-contact.vcf, outlook-contact.vcf, thunderbird-contact.vcf
- [x] tests/jscontact_clients.rs and tests/version.rs
- [x] Fold into cairn/spec (decoded-model, json-codecs), log, CHANGELOG
