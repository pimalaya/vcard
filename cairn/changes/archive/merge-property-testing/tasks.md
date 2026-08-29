---
cairn: tasks
change: merge-property-testing
---

# Tasks

- [x] Add `proptest` as a dev-dependency
- [x] Build the plain-data card model and the generator, and measure the collision rate
- [x] Encode the algebraic laws, one named test each
- [x] State the completeness law field by field, with its exclusions documented at their site
- [x] Write the naive reference merge and the differential against it
- [x] Drive the same layers from the corpus fixtures through the crate's edit layer
- [x] Add the `merge` fuzz target and run it from fuzz/shell.nix
- [x] Commit a failing `#[ignore]` reproduction for every defect found, each naming its write-up
- [x] cargo fmt, clippy, the full test suite and `--no-default-features`
- [x] Human review of the defects, one follow-up change each
- [x] Fold the delta into the spec, write the log entry, archive the change
