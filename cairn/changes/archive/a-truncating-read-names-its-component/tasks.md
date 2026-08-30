---
cairn: tasks
change: a-truncating-read-names-its-component
---

# Tasks

- [x] Give the value node whole-value readers: `decode`, `decode_list`, `decode_bytes`
- [x] Rename the component readers to name their component
- [x] Delete `decode_scalar_at` and `decode_bytes_at`, which cut twice
- [x] Give the node whole-value writers, and rename the component writers
- [x] Review every `..._at(0)` call site one at a time
- [x] Cover a value read past its first `;` and a component read past its first `,`
- [x] Cover a value read whole and written straight back
