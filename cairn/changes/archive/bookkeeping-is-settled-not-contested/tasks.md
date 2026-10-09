---
cairn: tasks
change: bookkeeping-is-settled-not-contested
---

# Tasks

- [x] `Merger::settles` and `later_revision` in tree/merge/merger.rs: a pair both sides restamped, and two additions, settle to the later line before any collision is sought.
- [x] `VcardTimestamp` reads a fraction of a second, and `to_unix_nanos` orders by it.
- [x] Tests: two content edits each restamping, both orders, across the basic and extended forms; two additions; a tie between two spellings; a 3.0 date against a date-time with a fraction; a stamp that does not read still contested.
- [x] The property tests' generator restamps a card each side edits, the reference merge settles the same way, and the completeness law expects the later line; the corpus edits, which write words onto a `REV`, check the fallback.
- [x] Audit RFC 6350 and RFC 9554 for other bookkeeping: none (`PRODID`, `CREATED`, `UID`, `CLIENTPIDMAP` stated in the spec).
- [x] Fold into merge, log, changelog.
