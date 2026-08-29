---
cairn: tasks
change: a-line-remembers-its-wire-shape
---

# Tasks

- [x] Add `tree::wire::VcardWire`, the fold, soft-break and skipped pieces
- [x] Record every artifact the tokeniser resolves, folds and soft breaks apart
- [x] Re-insert them from the byte serializer and from `Display`
- [x] Drop a shape an edit made stale, so an edited line goes out unfolded
- [x] Keep a card's trailing blank lines on `VcardCst`
- [x] Move the corpus laws from a fixpoint to byte identity
- [x] Feed the merge generator folded, soft-broken and blank-lined input
- [x] Align the README, the `src/lib.rs` header and the module headers
- [x] Fuzz both targets on the existing corpus
