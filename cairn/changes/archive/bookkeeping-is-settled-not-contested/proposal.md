---
cairn: change
id: bookkeeping-is-settled-not-contested
status: landed
created: 2026-10-09
---

# Bookkeeping is settled, not contested

## Why

ical-rs 0.6.0 settles `DTSTAMP`, `LAST-MODIFIED` and `SEQUENCE` instead of contesting them, after the Android calendar's conflict work found every edit made on both sides conflicting on them: each side restamps what it writes. Its merge spec names vcard-rs's missing twin, `REV`.

vcard-rs has the same noise. `REV` (RFC 6350 6.7.4) is the timestamp of the card's latest revision and a client rewrites it on every write, so two sides each editing a different field of one card always collided on `REV`, and the left side's stamp stood even when it was the older one. The report carried a conflict the user can do nothing with, beside the ones that matter.

## What

Two sides each writing a new `REV`, by a value change or by two additions over a base without one, settle on the later stamp without a conflict, whichever side wrote it, and a tie keeps the left side's. The later stamp's whole line stands: the one parameter `REV` takes is `VALUE`, which says how its own stamp reads (a vCard 3.0 `REV` may be a date, RFC 2426 3.6.4), so a stamp never lands under the other side's type.

A stamp reads as an instant across the forms the crate reads, the ones `VcardTimestamp::to_unix_seconds` documents, plus the fraction of a second RFC 2425 section 5.8.4 allows a vCard 3.0 date-time, which that reader rejected. `VcardTimestamp::to_unix_nanos` keeps that fraction so two stamps one second apart or less still order. A stamp that does not read falls back to the ordinary rules.

Audit RFC 6350 for another property deserving the rule, and state the decision in the spec.
