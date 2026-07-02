# Contributing guide

Thank you for investing your time in contributing to vcard-rs.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) for the conventions every repository shares (layering, `no_std`, modules, errors, code style, licensing, notes for AI agents);
3. the inline header documentation of all file, starting by `lib.rs`, for how the project is architectured;
3. this guide, for how to build, test and submit changes here.

## Development environment

The environment is managed by [Nix](https://nixos.org/download.html). `nix develop` spawns a shell with the right toolchain; every cargo command below assumes it (or prefix them with `nix develop --command`).

Without Nix, install a recent stable toolchain via [rustup](https://rust-lang.github.io/rustup/) (`rustup update`); the crate needs Rust matching the `rust-version` in [Cargo.toml](./Cargo.toml).

## Build

vcard-rs is a `#![no_std]` library (with `alloc`). Its core is dependency-free; the only dependencies sit behind opt-in features:

- `parser`: the byte-faithful content-line parser;
- `quoted-printable`, `base64`, `encoding`: content decoders for encoded text, inline binary and foreign character sets.

All four are on by default, so check both the full build and the bare core:

```sh
cargo build                                  # default features
cargo build --no-default-features            # dependency-free core, no std leak
cargo build --release --all-features
```

When touching feature gates or imports, build with and without each feature so no gated code leaks into the always-on core.

## Lint, test, audit

```sh
cargo test                                   # unit + integration + doc tests
cargo test --all-features                    # exercise every decoder path
cargo clippy --all-targets --all-features    # keep clean across the feature matrix
cargo fmt                                    # CI checks `cargo fmt --check`
```

Before opening a PR, make sure `cargo test`, `cargo clippy` and `cargo fmt --check` pass.

### Examples and benchmarks

The runnable snippets from the README live in [./examples](examples); run one with `cargo run --example <name>` (see [parse_edit_export](examples/parse_edit_export.rs), [strict_builder](examples/strict_builder.rs), [raw_builder](examples/raw_builder.rs)). The comparative parse benchmark runs with `cargo bench --bench parse`.

### The fixture corpus

Integration tests sweep a golden corpus of real-world cards under [tests/corpus](tests/corpus), organized one directory per source project, each with its own `ATTRIBUTION.md` so provenance and licensing stay per project. Adding a real card is the fastest way to turn a bug report into a regression test:

1. drop the card into the matching `tests/corpus/<project>/` directory (create a new one, with an `ATTRIBUTION.md`, for a new source);
2. bump the fixture count for that project in the corresponding sweep (`tests/corpus.rs`, `tests/github.rs` or `tests/rfc.rs`);
3. for a specific decoded assertion (parse this card, expect that value), add a focused case to `tests/github_cases.rs` instead;
4. run `cargo test`. Every parsed card must serialize to a fixpoint and decode without panicking; if it does not, you have found a bug, fix the code rather than the fixture.

New parser inputs are also worth handing to the fuzzer (see [fuzz/README.md](fuzz/README.md)).

## Commit style

vcard-rs follows the [conventional commits specification](https://www.conventionalcommits.org/en/v1.0.0/#summary). Keep the subject imperative and scoped; describe the *why* in the body when it is not obvious.
