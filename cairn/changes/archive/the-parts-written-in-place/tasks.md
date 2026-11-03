---
cairn: tasks
change: the-parts-written-in-place
---

- [x] Harness, baseline copy and differential crate refreshed from the tree after the-card-written-straight (df9565d); planted breaks in the old copy (commas inside quotes split, `URL` unknown to the kind lookup) caught on corpus files 2 and 10
- [x] Layout calibration: the baseline built from the scratchpad's path read 20 to 60% slower on the `to_bytes` rows than the same code built from the repository's; baselines now remap their path, and every result is checked in four layouts
- [x] Lever 1: the common line written into its slot from registers, its parameters filled where they land
- [x] Lever 2: properties decoded into their slots, values stored per kind, item lists sized once and filled in place
- [x] Lever 3: the kind lookups without the copy and the `memcmp`, checked against a search of every spelling
- [x] Lever 4: short pieces of `to_bytes` written by fixed-size moves; parameter values written in place by the text writer
- [x] Lever 5, in part: the line end, the value colon and the name's semicolon found by inlined 16-byte block scans; no simd-cpu scanner
- [ ] Lever 6: allocations, moved to the handoff
- [x] Each change kept only for its cycles (interleaved medians and minimums, four layouts for the final table), the dropped ones listed with their numbers
- [x] The checks of "Before landing a pass" (no `unsafe` changed, so no Miri run)
- [x] benches/README.md numbers, CHANGELOG, log, the-structure-scanned-once rewritten as the next pass's handoff
- [ ] Bench rows the handoff asked for (the corpus through `parse_many`, a folded `PHOTO` card, a 2.1 quoted-printable card, the decoders against base64 and quoted_printable): moved again to the-structure-scanned-once
- [ ] Commit after the user's yes
