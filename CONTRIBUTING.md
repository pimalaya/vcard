# Contributing guide

Thank you for investing your time in contributing to vcard-rs.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation, starting with src/lib.rs: it is the architecture document of this crate;
4. the docs/ folder for the development history and design notes.

Everything below documents only what differs from the Pimalaya standards.

## Build

vcard-rs is not an I/O library, so it has no coroutine, client or TLS layers. It is a no_std library (with alloc) whose core is dependency-free; every dependency sits behind an opt-in feature: parser (the byte-faithful content-line tree), quoted-printable, base64 and encoding (content decoders for encoded text, inline binary and foreign character sets), and jcard and jscontact (the JSON codecs). Everything under the tree module is gated on parser, while the decoded model is always available.

Check both the full build and the bare core, so gated code never leaks into the always-on core:

```sh
cargo build                          # default features
cargo build --no-default-features    # dependency-free model only, no std leak
cargo build --release --all-features
```

## The fixture corpus

Integration tests sweep a golden corpus of real-world cards under tests/corpus, one directory per source project, each with its own ATTRIBUTION.md so provenance and licensing stay per project. Adding a real card is the fastest way to turn a bug report into a regression test:

1. drop the card into the matching tests/corpus/<project>/ directory (create a new one, with an ATTRIBUTION.md, for a new source);
2. bump the fixture count for that project in the corresponding sweep (tests/corpus.rs, tests/github.rs or tests/rfc.rs);
3. for a specific decoded assertion (parse this card, expect that value), add a focused case to tests/github_cases.rs instead;
4. run cargo test. Every parsed card must serialize to a fixpoint and decode without panicking; if it does not, you have found a bug, so fix the code rather than the fixture.

New parser inputs are also worth handing to the fuzzer, described in [fuzz/README.md](./fuzz/README.md).

## Examples and benchmarks

The runnable programs from the docs live in [./examples](./examples); run one with cargo run --example followed by its name. The comparative parse benchmark runs with cargo bench --bench parse, and its methodology and current numbers are recorded in [docs/benchmarks.md](./docs/benchmarks.md).
