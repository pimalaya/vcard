---
cairn: change
id: an-edited-line-is-folded
status: landed
created: 2026-10-18
---

# An edited line is folded, keeping the folds before its edit

## Why

Most of fold-aware targeted patching is already here: an edit rewrites only the component it names and every other line of the card survives byte for byte, folds, blank lines, soft breaks, group prefixes and `X-` properties included (editing: edits are byte-preserving; parsing: round-trip fidelity, a-line-remembers-its-wire-shape, 2026-08-29).

The edited line itself is what is left. A length change drops its recorded shape, and the line goes out unfolded, which RFC 6350 3.2 permits but does not recommend: a `PHOTO` or a long `NOTE` edited in one character comes out as a single physical line of any length, every physical line of it changed. ical-rs folds such a line at 75 octets since rfc5545-contract-fixes (2026-10-09), and ical cairn/changes/an-edited-line-keeps-its-folds goes one step further, keeping the folds before the edit. This change brings vcard-rs to the same rule in one step, so the two crates fold an edited line alike, as they already share the wire shape design.

An external engine patches the target logical line fold-aware and leaves every other byte untouched; this crate does that for lines, and this change does it inside the line.

## What

- A line with no recorded shape (built, encoded from the model) is folded past 75 octets with a line break and a space, never inside a UTF-8 sequence, its line ending its own. A line pushed raw keeps the shape it was written in.
- A line an edit changed keeps every recorded fold before the first changed byte and is folded from the last kept fold onward, with its own folding whitespace and line ending.
- A `QUOTED-PRINTABLE` line is never folded: a fold after one of its `=` would read back as a soft break.
- A vCard 2.1 line is never refolded (decided 2026-11-03).
- A decoded card's text output folds as its encoding does.

## Decisions

- vCard 2.1 folds as RFC 822 does: a break goes before existing whitespace, which stays in the value on unfolding. Folding a 2.1 line as 3.0 and 4.0 are folded would add a space a 2.1 reader keeps, and folding it only at its own whitespace would not read back through this crate, which unfolds 2.1 as 3.0 does. A 2.1 line is left unfolded, as before (user, 2026-11-03). Whether the 2.1 unfold should keep the whitespace is a question of its own, needing real 2.1 exporters as evidence.
- The base64 question is moot: a fold is resolved away before the value reaches the decoder, and `decode_base64` skips line breaks and blanks since the-simd-codecs (2026-11-02).
