---
cairn: log
change: guidelines-alignment
landed: 2026-08-08
---

# Align with the Pimalaya guidelines

An audit against [.github/GUIDELINES.md](https://github.com/pimalaya/.github/blob/master/GUIDELINES.md), scope by scope, turned up violations in naming, cargo and inline documentation. They are fixed here.

**naming-005** (types live next to the code that owns them, never behind a private module plus a doc-inlined re-export). Three modules used the retired flatten. `tree::param::lens`, `tree::param::node`, `tree::prop::cardinality`, `tree::prop::lens`, `tree::prop::spec`, `tree::value::cursor` and `tree::value::node` are now public modules, and the module name is part of the public path: `tree::prop::lens::VcardPropLens`, `tree::value::cursor::VcardValueCursor`, and so on. The `#[doc(inline)] pub use ...::*` re-exports are gone.

**naming-002** (a module with code of its own is a sibling file, not a mod.rs). tree/param/mod.rs carried the shared `COMMON_PARAMS` constant, so it became tree/param.rs. tree/prop/mod.rs and tree/value/mod.rs are pure aggregators now that their re-exports are gone, so they stay where they are.

**naming-006, naming-007, naming-009** (the domain prefix is strict, and companions read largest scope first). `Codec` became `VcardCodec`, `Escaper` became `VcardEscaper`, `Valid` became `VcardValid`, and `VcardUnknownValue` became `VcardValueUnknown`. The `FromStr` errors were re-ordered onto `<Domain><Target><Verb><Ext>`, matching the `JmapEmailParseError` precedent: `ParseVcardPropKindError` became `VcardPropKindParseError`, and likewise for the param-kind, value-kind, version, jCard and JSContact errors.

The property and parameter lens markers keep their wire spelling (`FN`, `ADR`, `SORT_AS`) against naming-007. They are type-level keys naming a spec token, never handled as values, and a prefix would obscure the one thing they encode. The deviation is recorded in CONTRIBUTING.md, which is the deviations-only file, and flagged upstream so the guideline can grow a third exception.

**crate-004** (imports from the same crate are merged into a single `use`). The twelve parameter lens modules each carried two `use crate::` declarations; they are one now.

**crate-003** (a cargo feature is justified only when it pulls additional crates into the build). The `jscontact` feature was `["jcard"]` and pulled nothing of its own, so enabling it changed only which of our own code compiled. It is removed, and the JSContact conversion now ships under `jcard`, whose `serde_json` it already needed. That is the one change here that drops a published capability toggle.

**cargo-008** (every dependency pins `default-features = false` and enables only what it needs). `encoding_rs` now takes `alloc` explicitly, and the five dev-dependencies are pinned the same way, with `criterion` keeping only `cargo_bench_support`. **cargo-001** put the manifest blocks back in template order, and **cargo-007** gave the bench target an explicit `path`.

**inline-002, inline-003, inline-004**. The two undocumented `VcardParam::Unknown` fields gained docs; the blank lines separating the `Unknown` arms of `VcardParam` and `VcardValue` from their siblings are gone; the twenty-seven untagged `//` comments are now `NOTE:`; and every comment and doc line is back within eighty columns, apart from three that are a single unbreakable token or rustfmt-owned doctest code.

Capabilities moved: none by behaviour, but every capability file states the new public paths. This is an API-breaking rename with no change in what the crate does.
