---
cairn: tasks
change: the-card-written-straight
---

- [x] Scratch harness, baseline copy and differential crate refreshed from the tree after the-lines-built-in-place; the differential also compares `encode`, `to_string` and every property's encode under the three escapers, over decoded models salted with separators, escapes and carets; planted breaks in the old copy's 2.1 escape and `GEO` encoder caught
- [x] Allocation counts per corpus pass and per card (a counting allocator in the scratchpad)
- [x] The decoded card written straight to text, its scan vectorized and its buffer sized
- [x] Values read as text once: UTF-8 checked and a backslash searched once a value, plain splits without one
- [x] Parse: a constant placeholder line, its fields overwritten without a drop; the folds read once
- [x] `to_bytes`: a folded value laid out once from its node
- [x] Lever 5 (the user's yes, 2026-11-03): `VcardValue::Adr` boxed
- [x] Lever 6 (the user's yes, 2026-11-03): an ASCII proof in place of `str::from_utf8`, checked under Miri
- [x] Each change kept only for its cycles (interleaved medians and minimums), the dropped ones listed with their numbers
- [x] The checks of "Before landing a pass"
- [x] benches/README.md numbers, CHANGELOG, log, spec folded (decoded-model), the-structure-scanned-once rewritten as the next pass's handoff
- [ ] Bench rows the handoff asked for (the corpus through `parse_many`, a folded `PHOTO` card, a 2.1 quoted-printable card, the decoders against base64 and quoted_printable): moved again to the-structure-scanned-once
- [ ] Commit after the user's yes
