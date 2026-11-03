# Benchmarks

Single-card [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse` (source in [parse.rs](./parse.rs)), against ical_vcard 0.5, vparser 1.2, calcard 0.3 and vcard_parser 0.2. A dependency bump moves them, so re-run the bench after one.

The comparison is level-matched, so each group compares like with like: content-line parsers stop at a line tree like our `VcardCst::parse` step, while model parsers build a decoded object like our `parse + decode` step.

The figures below are the medians of three interleaved runs on 2026-11-03 (Zen 5, rustc 1.96.1, `taskset -c 4`) on a quiet machine. The previous figures, taken on 2026-11-02 at a load of about 24, read about 2.5 times these; compare deltas, not times, across the two.

## Parsing into content lines (no decoding)

| library | time | delta |
| --- | --- | --- |
| [`vparser`](https://crates.io/crates/vparser) | 0.28 µs | -52% |
| **vcard-rs** (`VcardCst::parse`) | **0.59 µs** | baseline |
| [`ical_vcard`](https://crates.io/crates/ical_vcard) | 2.08 µs | +250% |

## Parsing into a decoded model

| library | time | delta |
| --- | --- | --- |
| **vcard-rs** (`parse + decode`) | **1.81 µs** | baseline |
| [`calcard`](https://crates.io/crates/calcard) | 2.84 µs | +57% |
| [`vcard_parser`](https://crates.io/crates/vcard_parser) | 58.1 µs | +3117% |

## Reading the numbers

These are a ballpark, not a strict ranking: the libraries produce different representations (borrowed versus owned, shallow versus validating), so they do different amounts of work.

`vparser` is a zero-allocation pull tokenizer that builds no leaf or parameter structs, which is why it stays ahead at the content-line level; at the model level we lead `calcard`.

The `vcard` crate is builder-only and does not parse, so it is absent from both groups.

## This crate's pipeline

The same card through the bench's own rows, before and after the second speed pass ([the-card-written-straight](../cairn/log/2026-11-03-the-card-written-straight.md)), interleaved in the same session:

| step | before | after | delta |
| --- | ---: | ---: | ---: |
| `VcardCst::parse` | 749 ns | 594 ns | -21% |
| parse + `decode` | 2,216 ns | 1,807 ns | -18% |
| `decode` of a parsed card | 1,184 ns | 1,018 ns | -14% |
| `to_bytes` | 156 ns | 142 ns | -9% |
| parse + `to_bytes` | 935 ns | 782 ns | -16% |
| `to_string` of a decoded card | 3,569 ns | 363 ns | -90% |

## Cycles over the corpus

Criterion times one short card. The speed passes also count user-space cycles over tests/corpus (146 files, 437 KB, three folded `PHOTO`s holding 41% of the bytes) and over the same short card parsed 300 times, with `perf stat`, which the machine's load moves far less than wall time. The harness lives in [the speed-pass handoff](../cairn/changes/the-structure-scanned-once/proposal.md); the first pass's own table is in [its log](../cairn/log/2026-11-02-the-lines-built-in-place.md).

The second pass, medians of 11 interleaved runs a pass on a quiet machine:

| workload | before | after | delta |
| --- | ---: | ---: | ---: |
| corpus `parse` | 853k | 746k | -13% |
| corpus `parse` + `decode` | 2,736k | 2,190k | -20% |
| corpus `parse` + `to_bytes` | 1,162k | 992k | -15% |
| corpus `to_string` of the decoded cards | 5,719k | 326k | -94% |
| short card `parse` | 994k | 856k | -14% |
| short card `parse` + `decode` | 2,953k | 2,568k | -13% |
| short card `parse` + `to_bytes` | 1,229k | 1,078k | -12% |
| short card `to_string` | 4,958k | 519k | -90% |
