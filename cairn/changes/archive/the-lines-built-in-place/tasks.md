---
cairn: tasks
change: the-lines-built-in-place
---

- [x] Land the-simd-codecs first (landed 2026-11-02, uncommitted)
- [x] Baseline: criterion of the single card, corpus cycles (`parse` 1,551k, `decode` 4,889k, `bytes` 2,205k aligned), profiles of the three workloads, failed forwards counted
- [ ] Bench rows: the corpus through `parse_many` (rivals' coverage checked first), a 3.0 card with a folded 40 KB `PHOTO`, a 2.1 quoted-printable card, the two decoders against base64 0.23 and quoted_printable 0.5 (moved to the-structure-scanned-once; the scratch harness gained the short card instead)
- [ ] Allocation counts per corpus pass (moved to the-structure-scanned-once)
- [x] Scratch harness, baseline copy and differential crate (`vcard-old`), a planted break caught
- [x] Lever 1: lines written in place (and the wire shape's per-fold pushes)
- [x] Lever 2: a folded line built once
- [x] Lever 3: one pass over the head (the parameters in one walk; the colon scan and the head's UTF-8 check stay apart)
- [x] Lever 4: `decode` without the lossy path on UTF-8 values, the property kind matched exactly first
- [x] Lever 5: per-line allocations cut (line endings, the folded line's copies, the decoded property copies, the line list's first growths)
- [ ] Lever 6: kernel bench of a structural scanner against memchr on the corpus's line lengths; the scanner only if it wins (moved to the-structure-scanned-once)
- [x] Each lever kept only for its cycles (interleaved medians), the dropped ones listed with their numbers
- [x] The checks of "Before landing a pass" (the merge fuzz target found a crash the baseline shares, logged)
- [x] benches/README.md numbers (README.md carries none), log, the next pass's handoff written (the-structure-scanned-once)
- [ ] Commit after the user's yes
