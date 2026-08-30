# Contributing guide

Thank you for investing your time in contributing to vcard-rs.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation, starting with src/lib.rs: it is the architecture document of this crate;
4. the [cairn/](./cairn) folder for the development history and living plans (the Cairn convention: spec/, changes/, log/).

Everything below documents only what differs from the Pimalaya standards.

## Deviation: wire-spelled lens markers

The property and parameter lens markers are spelled exactly as their wire token (`FN`, `ADR`, `SORT_AS`, `TYPE`), against naming-007, which asks every public item to carry the `Vcard` domain prefix. Every other public item does carry it.

They are type-level keys naming a spec token, written only inside a turbofish (`card.prop::<FN>()`), never constructed and never handled as values.

Prefixing them would push the wire name, the one thing they encode, to the end of a longer identifier, and make the call site read less like the card it addresses. Two of them (`SORT_AS`, `SORT_STRING`) also carry `#[allow(non_camel_case_types)]` for the same reason.

The discrepancy is flagged upstream, so the guideline can grow a third exception for spec-token type-level keys.

## Deviation: the jscontact feature

The `jscontact` feature is `["jcard"]` and pulls no crate of its own, against crate-003, which asks that a feature exist only where it changes the crate set. Every other feature here does pull one: `parser` takes `memchr`, `jcard` takes `serde_json`, and the three content decoders take one small crate each.

It is kept because a cargo feature is also a discovery surface, not only a build switch: a reader looking for JSContact support reads the feature list, and folding it into `jcard` would leave the RFC 9555 conversion undiscoverable from the manifest.

The alias also states the real dependency, `jscontact` implying `jcard` since the conversion reuses jCard syntax for its vCardProps and vCardParams escape hatches. It costs a name and no build weight.

## Cairn

This repository follows [Cairn](https://github.com/pimalaya/cairn): a living spec, reviewable change proposals and a dated log, kept next to the code.

Non-trivial work starts with a change folder under cairn/changes, and nothing behavioural is done until its delta is folded into cairn/spec and an entry is appended to cairn/log. The activation stanza is [AGENTS.md](./AGENTS.md).

## Build

vcard-rs is not an I/O library, so it has no coroutine, client or TLS layers. It is a no_std library (with alloc) whose core is dependency-free, every dependency sitting behind a feature.

`parser` brings the byte-faithful content-line tree; `quoted-printable`, `base64` and `encoding` decode encoded text, inline binary and foreign character sets; `jcard` and `jscontact` are the JSON codecs.

Everything under the tree module is gated on `parser`, while the decoded model is always available.

Check both the full build and the bare core, so gated code never leaks into the always-on core:

```sh
cargo build                          # default features
cargo build --no-default-features    # dependency-free model only, no std leak
cargo build --release --all-features
```

## The fixture corpus

Integration tests sweep a golden corpus of real-world cards under tests/corpus, one directory per source project, each with its own ATTRIBUTION.md so provenance and licensing stay per project. Adding a real card is the fastest way to turn a bug report into a regression test:

1. drop the card into the matching tests/corpus/<project>/ directory (create a new one, with an ATTRIBUTION.md, for a new source);
2. bump the fixture count for that project in the corresponding sweep (tests/corpus.rs, tests/calcard.rs, tests/github.rs or tests/rfc.rs);
3. for a specific decoded assertion (parse this card, expect that value), add a focused case to tests/github_cases.rs instead;
4. run cargo test. Every parsed card must serialize to a fixpoint and decode without panicking; if it does not, you have found a bug, so fix the code rather than the fixture.

New parser inputs are also worth handing to the fuzzer, described in [fuzz/README.md](./fuzz/README.md).

## Examples and benchmarks

The runnable programs from the docs live in [./examples](./examples); run one with cargo run --example followed by its name. The comparative parse benchmark runs with cargo bench --bench parse, and its methodology and current numbers are recorded in [benches/README.md](./benches/README.md).
