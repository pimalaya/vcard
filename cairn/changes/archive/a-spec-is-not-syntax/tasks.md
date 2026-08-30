---
cairn: tasks
change: a-spec-is-not-syntax
---

- [x] Move `prop::spec`, `prop::cardinality` and `param::COMMON_PARAMS` to the root
- [x] Split the 48 markers: type and spec at the root, lens and cursor under `tree`
- [x] Move `builder` and `validator` to the root
- [x] Move the `VcardValid` to `VcardCst` bridge into `tree::codec::encode`
- [x] Drop `parser` from the `jcard` feature
- [x] Repoint every `tree::prop::<name>::<MARKER>` path, in the crate, the tests, the examples and the benches
- [x] Split each marker's module header over its two halves
- [x] `cargo fmt`, `clippy --all-targets`, `test --all-features`, `build --no-default-features`, `build --no-default-features --features jscontact`, `doc`
