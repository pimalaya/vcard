---
cairn: log
change: bookkeeping-is-settled-not-contested
landed: 2026-10-09
---

# Bookkeeping is settled, not contested

The twin of the ical-rs 0.6.0 rule of the same name. Every card both sides edited conflicted on `REV`, each side having restamped what it wrote, and the left side's stamp stood even when it was the older one. `REV` records when the card was revised (RFC 6350 6.7.4), not what it says.

`Merger::settles` (tree/merge/merger.rs) runs before any right-side action on a pair replays: when both sides changed the value of one `REV`, the later stamp's whole line stands and nothing is reported, the right side's line replacing the left's only where it is strictly later. Two additions of `REV` over a base without one settle the same way in `apply_added`. The whole line rather than the value alone, because `VALUE`, the one parameter `REV` takes, says how its own stamp reads: a vCard 3.0 date (RFC 2426 3.6.4) against a date-time would otherwise put the date-time under `VALUE=date`.

`later_revision` reads both stamps through `VcardTimestamp`, so the basic and extended forms, offsets, reduced precision and a date alone compare as instants. The reader learnt the fraction of a second RFC 2425 section 5.8.4 allows a 3.0 date-time, after a comma or a full stop, which it used to reject; `to_unix_seconds` drops it and the new `to_unix_nanos` keeps it. Two sides writing the same bytes still agree through the ordinary path, and a stamp that does not read leaves the pair to the ordinary rules.

Audited for other bookkeeping: none. `PRODID` carries no order to settle by, `CREATED` is written once, `UID` is identity, `CLIENTPIDMAP` is synchronisation data. The spec states each.

The property tests (tests/merge.rs) give two base cards in three a `REV`, restamp it on every side that edits the card unless the side wrote it itself, draw stamps in both forms and with an offset, a tie among them, and add two `REV`s to the addition pool. The reference merge settles the same way and the completeness law expects the later line. The corpus edits write words onto a `REV`, which checks the fallback. Mutating each of the two settle paths, or flipping the comparison, fails the laws.

Capabilities moved: merge, decoded-model.
