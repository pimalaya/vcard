# Benchmarks

Single-card [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse` (source in [parse.rs](./parse.rs)), against ical_vcard 0.5, vparser 1.2, calcard 0.3 and vcard_parser 0.2. A dependency bump moves them, so re-run the bench after one.

The comparison is level-matched, so each group compares like with like: content-line parsers stop at a line tree like our `VcardCst::parse` step, while model parsers build a decoded object like our `parse + decode` step.

## Parsing into content lines (no decoding)

| library | time | delta |
| --- | --- | --- |
| [`vparser`](https://crates.io/crates/vparser) | 0.48 µs | -67% |
| **vcard-rs** (`VcardCst::parse`) | **1.46 µs** | baseline |
| [`ical_vcard`](https://crates.io/crates/ical_vcard) | 3.80 µs | +160% |

## Parsing into a decoded model

| library | time | delta |
| --- | --- | --- |
| **vcard-rs** (`parse + decode`) | **4.72 µs** | baseline |
| [`calcard`](https://crates.io/crates/calcard) | 4.84 µs | +2% |
| [`vcard_parser`](https://crates.io/crates/vcard_parser) | 83.1 µs | +1660% |

## Reading the numbers

These are a ballpark, not a strict ranking: the libraries produce different representations (borrowed versus owned, shallow versus validating), so they do different amounts of work.

`vparser` is a zero-allocation pull tokenizer that builds no leaf or parameter structs, which is why it stays ahead at the content-line level; at the model level we are on par with `calcard`.

The `vcard` crate is builder-only and does not parse, so it is absent from both groups.
