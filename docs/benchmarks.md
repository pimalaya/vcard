# Benchmarks

Single-card [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse` (source in [benches/parse.rs](../benches/parse.rs)).

The comparison is level-matched, so each group compares like with like: content-line parsers stop at a line tree like our `VcardCst::parse` step, while model parsers build a decoded object like our `parse + decode` step.

## Parsing into content lines (no decoding)

| library | time | delta |
| --- | --- | --- |
| [`vparser`](https://crates.io/crates/vparser) | 0.57 µs | -61% |
| **vcard-rs** (`VcardCst::parse`) | **1.47 µs** | — |
| [`ical_vcard`](https://crates.io/crates/ical_vcard) | 2.82 µs | +92% |

## Parsing into a decoded model

| library | time | delta |
| --- | --- | --- |
| [`calcard`](https://crates.io/crates/calcard) | 4.40 µs | -1% |
| **vcard-rs** (`parse + decode`) | **4.46 µs** | — |
| [`vcard_parser`](https://crates.io/crates/vcard_parser) | 87.8 µs | +1869% |

## Reading the numbers

These are a ballpark, not a strict ranking: the libraries produce different representations (borrowed versus owned, shallow versus validating), so they do different amounts of work. `vparser` is a zero-allocation pull tokenizer that builds no leaf or parameter structs, which is why it stays ahead at the content-line level; at the model level we are on par with `calcard`. The `vcard` crate is builder-only and does not parse, so it is absent from both groups.
