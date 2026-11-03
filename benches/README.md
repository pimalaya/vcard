# Benchmarks

Single-card [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse` (source in [parse.rs](./parse.rs)), against ical_vcard 0.5, vparser 1.2, calcard 0.3 and vcard_parser 0.2. A dependency bump moves them, so re-run the bench after one.

The comparison is level-matched, so each group compares like with like: content-line parsers stop at a line tree like our `VcardCst::parse` step, while model parsers build a decoded object like our `parse + decode` step.

The figures below are the medians of three interleaved runs on 2026-11-03 (Zen 5, rustc 1.96.1, `taskset -c 4`) with nothing else on that core.

## Parsing into content lines (no decoding)

| library | time | delta |
| --- | --- | --- |
| [`vparser`](https://crates.io/crates/vparser) | 0.29 µs | -42% |
| **vcard-rs** (`VcardCst::parse`) | **0.50 µs** | baseline |
| [`ical_vcard`](https://crates.io/crates/ical_vcard) | 2.12 µs | +326% |

## Parsing into a decoded model

| library | time | delta |
| --- | --- | --- |
| **vcard-rs** (`parse + decode`) | **1.55 µs** | baseline |
| [`calcard`](https://crates.io/crates/calcard) | 2.92 µs | +88% |
| [`vcard_parser`](https://crates.io/crates/vcard_parser) | 59.6 µs | +3734% |

## Reading the numbers

These are a ballpark, not a strict ranking: the libraries produce different representations (borrowed versus owned, shallow versus validating), so they do different amounts of work.

`vparser` is a zero-allocation pull tokenizer that builds no leaf or parameter structs, which is why it stays ahead at the content-line level; at the model level we lead `calcard`.

The `vcard` crate is builder-only and does not parse, so it is absent from both groups.

## This crate's pipeline

The same card through the bench's own rows, before and after the third speed pass ([the-parts-written-in-place](../cairn/log/2026-11-03-the-parts-written-in-place.md)), interleaved in the same session:

| step | before | after | delta |
| --- | ---: | ---: | ---: |
| `VcardCst::parse` | 602 ns | 497 ns | -17% |
| parse + `decode` | 1,893 ns | 1,554 ns | -18% |
| `decode` of a parsed card | 1,042 ns | 916 ns | -12% |
| `to_bytes` | 147 ns | 133 ns | -10% |
| parse + `to_bytes` | 795 ns | 658 ns | -17% |
| `to_string` of a decoded card | 376 ns | 344 ns | -9% |

## Cycles over the corpus

Criterion times one short card. The speed passes also count user-space cycles over tests/corpus (146 files, 437 KB, three folded `PHOTO`s holding 41% of the bytes) and over the same short card parsed 300 times, with `perf stat`, which the machine's load moves far less than wall time. The harness lives in [the speed-pass handoff](../cairn/changes/the-structure-scanned-once/proposal.md); the earlier passes' tables are in their logs ([the first](../cairn/log/2026-11-02-the-lines-built-in-place.md), [the second](../cairn/log/2026-11-03-the-card-written-straight.md)).

The third pass, medians of 7 interleaved runs, the before and after builds laid out alike (the last column: the mean delta over four code layouts):

| workload | before | after | delta | four layouts |
| --- | ---: | ---: | ---: | ---: |
| corpus `VcardCst::parse` | 699k | 578k | -17% | -18% |
| corpus `parse` (`parse_many`) | 744k | 623k | -16% | -17% |
| corpus `parse` + `decode` | 2,202k | 2,061k | -6% | -6% |
| corpus `parse` + `to_bytes` | 982k | 849k | -14% | -13% |
| corpus `to_string` of the decoded cards | 322k | 314k | -3% | -3% |
| short card `VcardCst::parse` | 849k | 647k | -24% | -25% |
| short card `parse` | 857k | 674k | -21% | -21% |
| short card `parse` + `decode` | 2,618k | 2,256k | -14% | -13% |
| short card `parse` + `to_bytes` | 1,071k | 878k | -18% | -18% |
| short card `to_string` | 520k | 461k | -11% | -11% |
