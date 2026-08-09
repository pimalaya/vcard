---
cairn: tasks
change: jscontact-rfc-object-types
---

# Tasks

- [x] Rename the five resource `@type` literals in src/jscontact.rs to their RFC 9553 §2.6 names
- [x] Update the doc comments and test fixtures that spell the draft names
- [x] Add a test pinning each collection's exported `@type`
- [x] Confirm the import side still round-trips a Card written with either spelling
- [x] cargo fmt, clippy and the full test suite
- [x] Fold the delta into the spec, write the log entry, archive the change, update CHANGELOG.md
