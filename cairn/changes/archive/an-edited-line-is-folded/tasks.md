---
cairn: tasks
change: an-edited-line-is-folded
---

- [x] Decide the 2.1 rule: left unfolded (user, 2026-11-03)
- [x] src/tree/wire.rs: fold a shapeless line at 75 octets, UTF-8 safe, its own ending (ical-rs's `FOLD_OCTETS` rule); cut a shape at the first changed offset and refold the tail with the line's own whitespace
- [x] Every cursor edit (the value cursor, the list cursor, the typed property cursors) goes through `VcardLine::edit_value`, which cuts the shape at the first changed byte
- [x] A decoded card's text output and a line's `Display` fold as `to_bytes` does
- [x] Tests: a long built line; the delta's scenario; a two-octet character at the cut; a same-length edit across an old fold; a `QUOTED-PRINTABLE` line untouched; a 2.1 card; a bare LF line; an unedited fold before a raw 8-bit byte; the corpus sweeps still byte-identical; the merge and parse fuzz targets
- [ ] Same rule and tests as ical cairn/changes/an-edited-line-keeps-its-folds, cross-checked (left to that change: vcard-rs alone, user 2026-11-03)
- [x] Update the src/tree/wire.rs module docs ("an edited line is written unfolded")
- [x] Fold parsing.md, log, CHANGELOG
