# Fuzzing

Coverage-guided fuzzing of the vCard parser with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer). The `parse` target checks two oracles: parsing arbitrary bytes never panics, and whatever parses serializes to a byte-stable fixpoint.

cargo-fuzz needs a nightly toolchain (for the `-Z` sanitizer flags). On NixOS, get both from the dedicated `fuzz/shell.nix` (nightly via fenix plus cargo-fuzz, no rustup or nix-ld needed):

```sh
nix-shell fuzz/shell.nix --run "cargo fuzz run parse"
```

libFuzzer saves every interesting new input into `fuzz/corpus/parse/` (gitignored), and any crash into `fuzz/artifacts/parse/`. Do not pass `tests/corpus` as the corpus directory: libFuzzer treats the first corpus directory as writable and would dump generated inputs into the curated fixtures. To warm-start coverage from the real cards, seed the fuzz corpus once with a copy:

```sh
mkdir -p fuzz/corpus/parse && cp tests/corpus/*/*.vcf fuzz/corpus/parse/
```

Off NixOS, `cargo install cargo-fuzz` and a nightly toolchain give the same `cargo fuzz run parse`.

For a quick, dependency-free smoke test on stable Rust (no libFuzzer), use the example instead:

```sh
cargo run --release --example fuzz_smoke -- 1000000
```
