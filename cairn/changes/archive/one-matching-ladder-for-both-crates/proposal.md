---
cairn: change
id: one-matching-ladder-for-both-crates
status: landed
created: 2026-08-29
---

# One matching ladder, the same in both crates

## Why

The twins solved one problem two ways. ical-rs addresses a property by an identity where iCalendar gives it one and by position only where it does not. vcard-rs matched by `PID`, then by equality, then by position, and had no notion of a property whose value names a thing outside the card.

vCard has such properties, and more of them than iCalendar. An `EMAIL` is a mailbox, a `TEL` a phone line, a `MEMBER` another card. Matching two of them by position writes one person's edit onto another the moment a side reorders or removes one, which is the defect ical-rs repaired.

The ladder is what both crates should share, so the difference between the two formats is data rather than a second design nobody can compare.

## What

Give vcard-rs the natural identity rung, between the `PID` rung and the equality rung, and guard the position rung so an identified instance never meets an unidentified one. Carry over the two other rules ical-rs learned from its fuzzer.

Keep `PID` above the natural identity: `PID` is metadata, so a vCard identity survives a value change and a rename stays a rename, which is the one place the twins genuinely differ.

Write the comparison discipline into the spec: matching normalises, writing is exact. Write down too that the positional rung is safe only because the base card is never mutated.
