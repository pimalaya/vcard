---
cairn: delta
change: bookkeeping-is-settled-not-contested
---

## ADDED Requirements

### Requirement: Bookkeeping is settled, not contested
Into merge. Two sides each writing a new `REV`, by a value change or by two additions, is no collision: the later stamp's whole line stands, whichever side wrote it, a tie keeping the left side's, and nothing is reported. A stamp reads as an instant across the forms the crate reads, a vCard 3.0 fraction of a second included. A stamp that does not read is contested as any other value. No other RFC 6350 property is bookkeeping in that sense. ical-rs states the twin rule.

### Requirement: A timestamp orders as an instant
Into decoded-model. `VcardTimestamp::to_unix_seconds` and the new `to_unix_nanos` read the forms the crate accepts, a vCard 3.0 fraction of a second included, the second keeping it.

## MODIFIED Requirements

### Requirement: Every change either lands or is reported
In merge: a revision stamp the bookkeeping rule settled is the one change that neither lands nor is reported.

## REMOVED Requirements
