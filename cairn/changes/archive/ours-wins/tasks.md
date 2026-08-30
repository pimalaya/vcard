---
cairn: tasks
change: ours-wins
---

- [x] Remove `prefer` from `VcardMerge` and delete `VcardMergeSide`
- [x] Keep the left side's value where both sides wrote one, and still report the collision
- [x] Collapse every `replaces` call site to recording the conflict, and delete `replaces`
- [x] Delete what became unreachable: the two replace-where-it-stood paths, `param_position`, `is_removal`
- [x] Verify an update still beats a removal, at property and at parameter granularity
- [x] Restate the rule in the module header, on `VcardMergeConflict`, and in the spec, as ours and theirs
- [x] Drop the preference from the integration tests and the fuzz target
- [x] Fold the spec and log the change
