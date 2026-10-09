---
cairn: change
id: a-meaning-change-contests-a-value-change
status: landed
created: 2026-10-09
---

# A meaning change contests a value change

## Why

ical-rs 0.6.0 made a parameter that changes what a value means contest a concurrent value change (its change `a-zone-change-contests-a-time-change`), after a merge wrote a time neither side wrote: one side's zone on the other side's time.

vcard-rs had the same gap, wider: no parameter contested a value change at all, `VALUE` included. Base `NOTE;LANGUAGE=en:Good morning`, one side `NOTE;LANGUAGE=en:Good evening`, the other `NOTE;LANGUAGE=fr:Bonjour`: the merge wrote `NOTE;LANGUAGE=fr:Good evening`, English text labelled French, and reported only the text. Base `BDAY:19850412`, one side moving the day, the other declaring `CALSCALE=x-hebrew`: the moved Gregorian date came out read in the other calendar, and nothing was reported.

## What

A change to a parameter that says what its value means collides with the other side's value-level action (whole value, component or list item), both ways, and the left side's line stands. Each pair is reported, so the count is the same whichever side is called left.

Audit RFC 6350 section 5, RFC 9554 and the vCard 2.1 and 3.0 parameters the crate reads for which ones say what the value means: `VALUE`, `LANGUAGE`, `CALSCALE`, `MEDIATYPE`, `PHONETIC`, `SCRIPT`, `ENCODING`, `CHARSET` and the bare 2.1 `QUOTED-PRINTABLE`. A parameter describing the thing a value names (`TYPE`, `PREF`, `ALTID`, `PID`, `SORT-AS`, `GEO`, `TZ`, `LABEL`, the other RFC 9554 ones) still merges beside a value change.

Two meaning actions contest nothing: one both sides made alike, since the value either wrote beside it was written for it, and a parameter removal the other side's update of that parameter beats, since the update is the meaning that stands.
