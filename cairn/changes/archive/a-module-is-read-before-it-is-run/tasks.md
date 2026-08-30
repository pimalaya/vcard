---
cairn: tasks
change: a-module-is-read-before-it-is-run
---

- [x] Flatten `tree::vcard` into `tree::builder` and `tree::validator`
- [x] Split `tree/merge.rs` into `compare`, `diff`, `instance`, `matching`, `merger` and `slot`
- [x] Attach the merge's free functions to `Instance`, `Matching`, `Diff`, `Merger`, `Slot` and the nodes they compare
- [x] Split `jcard.rs` into `export`, `import` and `datetime`
- [x] Split `jscontact.rs` into `export`, `import`, `params`, `date` and `pointer`
- [x] Distribute each module header over its submodules
- [x] Drop the per-field docs on private structs and enums, and spell `insts` out
- [x] Audit every `NOTE:` in `src/`, keeping only what the code cannot say
- [x] `cargo fmt`, `clippy --all-targets`, `test --all-features`, `build --no-default-features`, `doc`
