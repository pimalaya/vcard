---
cairn: tasks
change: the-structure-scanned-once
---

The handoff stays active across passes; each pass lands under its own change id and checks its own tasks there (the third: [the-parts-written-in-place](../archive/the-parts-written-in-place/tasks.md)). What is left for the next:

- [ ] Bench rows: the corpus through `parse_many` (rivals' coverage checked first), a 3.0 card with a folded 40 KB `PHOTO`, a 2.1 quoted-printable card, the two decoders against base64 0.23 and quoted_printable 0.5
- [ ] Baseline copy, harness (four layouts) and differential crate refreshed from the tree after the-parts-written-in-place, a planted break caught
- [ ] Lever 1: allocations, the large line and property lists and the small chunks a card frees, measured on parse plus decode
- [ ] Lever 2: the decoded parameter filled in place, its value list never copied back
- [ ] Lever 3: the per-card moves, the card built where its caller keeps it
- [ ] Lever 4: the text writer's `memcpy` calls
- [ ] Lever 5: a wider scanner on simd-cpu only if a kernel bench beats the 16-byte block scans; the parameter split
- [ ] Lever 6: the smaller items (`VcardPropName::deref`, the lookups' compare loops, the shrinking `realloc`s)
- [ ] Each lever kept only for its cycles (interleaved medians and minimums, four layouts for the pass's table), the dropped ones listed with their numbers
- [ ] The checks of "Before landing a pass"
- [ ] benches/README.md numbers, CHANGELOG, a log under the pass's own change id, this proposal rewritten, commit after the user's yes
