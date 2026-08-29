# Fuzzing

Coverage-guided fuzzing with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer). Two targets. The `parse` target checks two oracles: parsing arbitrary bytes never panics, and whatever parses serializes to a byte-stable fixpoint. The `merge` target carves three cards out of one input, so the mutator produces related copies rather than three unrelated cards, and checks the three-way merge laws: the merged card reparses to a fixpoint, a line all three copies carry keeps its bytes, an untouched side contributes nothing, and two identical edits are not a disagreement. Its property-testing twin is tests/merge.rs.

cargo-fuzz needs a nightly toolchain (for the `-Z` sanitizer flags). On NixOS, get both from the dedicated fuzz/shell.nix (nightly via fenix plus cargo-fuzz, no rustup or nix-ld needed):

```sh
nix-shell fuzz/shell.nix --run "cargo fuzz run parse"
nix-shell fuzz/shell.nix --run "cargo fuzz run merge"
```

libFuzzer saves every interesting new input into fuzz/corpus/<target>/ (gitignored), and any crash into fuzz/artifacts/<target>/. Do not pass tests/corpus as the corpus directory: libFuzzer treats the first corpus directory as writable and would dump generated inputs into the curated fixtures. To warm-start coverage from the real cards, seed the fuzz corpus once with a copy:

```sh
mkdir -p fuzz/corpus/parse fuzz/corpus/merge
cp tests/corpus/*/*.vcf fuzz/corpus/parse/
cp tests/corpus/*/*.vcf fuzz/corpus/merge/
```

Off NixOS, `cargo install cargo-fuzz` and a nightly toolchain give the same `cargo fuzz run`.
