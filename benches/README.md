# Benchmarks

Single-card [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse` (source in [parse.rs](./parse.rs)), against ical_vcard 0.5, vparser 1.2, calcard 0.3 and vcard_parser 0.2. A dependency bump moves them, so re-run the bench after one.

The comparison is level-matched, so each group compares like with like: content-line parsers stop at a line tree like our `VcardCst::parse` step, while model parsers build a decoded object like our `parse + decode` step.

The figures below are the medians of three interleaved runs on 2026-11-02 (Zen 5, rustc 1.96.1, `taskset -c 4`), taken while the machine ran other work at a load of about 24: every library's absolute time runs about 2.5 times its quiet figure, so read the deltas, which held within 6% between runs for the rivals.

## Parsing into content lines (no decoding)

| library | time | delta |
| --- | --- | --- |
| [`vparser`](https://crates.io/crates/vparser) | 1.00 µs | -57% |
| **vcard-rs** (`VcardCst::parse`) | **2.32 µs** | baseline |
| [`ical_vcard`](https://crates.io/crates/ical_vcard) | 6.29 µs | +171% |

## Parsing into a decoded model

| library | time | delta |
| --- | --- | --- |
| **vcard-rs** (`parse + decode`) | **6.64 µs** | baseline |
| [`calcard`](https://crates.io/crates/calcard) | 8.56 µs | +29% |
| [`vcard_parser`](https://crates.io/crates/vcard_parser) | 152.6 µs | +2198% |

## Reading the numbers

These are a ballpark, not a strict ranking: the libraries produce different representations (borrowed versus owned, shallow versus validating), so they do different amounts of work.

`vparser` is a zero-allocation pull tokenizer that builds no leaf or parameter structs, which is why it stays ahead at the content-line level; at the model level we lead `calcard`.

The `vcard` crate is builder-only and does not parse, so it is absent from both groups.

## Cycles over the corpus

Criterion times one short card. The speed passes also count user-space cycles over tests/corpus (146 files, 437 KB, three folded `PHOTO`s holding 41% of the bytes) and over the same short card parsed 300 times, with `perf stat`, which the machine's load moves far less than wall time. The harness lives in [the speed-pass handoff](../cairn/changes/the-structure-scanned-once/proposal.md).

| workload | before the first pass | after it | delta |
| --- | ---: | ---: | ---: |
| corpus `parse` | 2,427k | 1,453k | -40% |
| corpus `parse` + `decode` | 7,190k | 4,092k | -43% |
| corpus `parse` + `to_bytes` | 3,529k | 1,982k | -44% |
| short card `parse` | 2,823k | 1,819k | -36% |
| short card `parse` + `decode` | 7,912k | 5,172k | -35% |
| short card `parse` + `to_bytes` | 3,773k | 2,219k | -41% |
