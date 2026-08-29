---
cairn: change
id: a-merge-states-its-sides-and-its-winner
status: landed
created: 2026-08-29
---

# A merge states its sides, and its winner

## Why

`merge(base, left, right)` is three positional arguments, and a caller that wants to say anything else about the merge cannot. ical-rs took the other shape, a struct whose fields are named, and grew two options on it without breaking a caller. The two crates are twins and a reader compares them, so the better shape should win in both.

One of those options is missing here and is not a design difference. ical-rs lets a caller say which side wins a collision, apart from which side supplies the baseline bytes, because the two are different questions: the baseline decides whose folding and whose ordering survive untouched, the winner decides whose value survives where two people wrote different things. Here they are one, and tCard puts local on the left because that is the only way to keep the local value. Its sibling tCal puts local on the right and states a preference, so the two tools reconciling the same shape of divergence ended up with opposite conventions.

Organiser authority is deliberately not ported: it is RFC 5546, and vCard has no organiser.

## What

Add `VcardMerge { base, left, right, prefer }` with a `merge(self)` method, mirroring `IcalMerge`. Add `VcardMergeSide`, mirroring `IcalMergeSide`, defaulting to `Left` so nothing moves.

Keep the free `merge` function as a deprecated shim over the struct, so tCard and neverest keep building while they migrate.
