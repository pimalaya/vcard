---
cairn: log
change: cairn-adoption
landed: 2026-08-08
---

# Adopt Cairn, retiring docs/

docs/ held a README index, a design.md mixing current architecture with the rationale behind it, and a benchmarks.md. Cairn splits those axes, so this repository now keeps cairn/spec/ (current truth, one file per capability), cairn/changes/ (in-flight proposals) and cairn/log/ (dated history), with cairn.toml as the root marker and AGENTS.md (plus CLAUDE.md) as the activation stanza.

The migration was mechanical. The architecture summarised in docs/design.md and in the src/lib.rs header was seeded into six capability files: `parsing`, `decoded-model`, `editing`, `conformance`, `content-encodings` and `json-codecs`. That is a backfill Cairn normally discourages, done once here because the behaviour it describes already exists and later changes need something to state their deltas against. docs/benchmarks.md moved to benches/README.md, next to the code it documents, matching fuzz/README.md. docs/ was deleted.

The rejected designs recorded in docs/design.md are rationale, not current truth, so they land here rather than in the spec. A root generic parameterized by version was rejected: the version is a value, not a type, and threading it through every type buys nothing the runtime indicator does not. A second, stricter data model splitting a property into strict and lossy variants was rejected: validity and lossiness are orthogonal, so that splits the wrong axis. A generic lossy wrapper across prop, param and value was rejected: it fits only the property name, since value and param are payload unions whose `Unknown` already carries structured raw data. Modeling the 2.1 inline `AGENT` recursively was rejected as a denial-of-service risk, so `AGENT` stays raw text with an opt-in single-level re-parse helper. Core-side quoted-printable decoding was tried and reverted: it ran a lossy UTF-8 conversion that silently destroyed foreign-charset bytes.

The src/lib.rs header remains the entry point for the code. The spec is the behavioural truth behind it, and the forcing rule now applies: a behaviour change is not done until its delta is folded into the spec and an entry is appended here.

Capabilities moved: none. This change moved documentation, not behaviour.
