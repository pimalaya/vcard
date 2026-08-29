---
cairn: log
change: merge-property-testing
landed: 2026-08-29
---

# The three-way merge earns property, differential and fuzz coverage

Added tests/merge.rs, three layers over one plain-data model of a card: the algebraic laws, the completeness law stated field by field, and a differential against a naive reference merge that reconciles by the documented rules. The same layers run over the corpus fixtures through the crate's own edit layer, and fuzz/fuzz_targets/merge.rs carves three related cards out of one libFuzzer input. `proptest` joined the dev-dependencies, with committed regression seeds.

No behaviour changed. The suite found thirteen defects, each committed as an `#[ignore]` reproduction naming its write-up, each to be repaired by its own change.

Spec updated: `merge` (ADDED: every change either lands or is reported; the merge obeys its algebraic laws).
