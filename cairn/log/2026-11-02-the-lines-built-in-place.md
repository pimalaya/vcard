---
cairn: log
change: the-lines-built-in-place
landed: 2026-11-02
---

# The lines built in place

The first speed pass over the parser and the decoded model, taken from the handoff [the-lines-built-in-place](../changes/archive/the-lines-built-in-place/proposal.md), with no behaviour change. The baseline is the working tree after [the-simd-codecs](./2026-11-02-the-simd-codecs.md), uncommitted at the time.

## Results

User-space cycles a pass (`perf stat -e cycles:u` at 20 and 120 passes, the difference taken, medians of 7 interleaved runs of each binary, both built with `-align-all-functions=6 -align-loops=64`, `taskset -c 4`), over tests/corpus and over the criterion card parsed 300 times:

| workload | before | after | delta |
| --- | ---: | ---: | ---: |
| corpus `parse` | 2,427k | 1,453k | -40% |
| corpus `decode` (parse then `decode`) | 7,190k | 4,092k | -43% |
| corpus `bytes` (parse then `to_bytes`) | 3,529k | 1,982k | -44% |
| card `parse` | 2,823k | 1,819k | -36% |
| card `decode` | 7,912k | 5,172k | -35% |
| card `bytes` | 3,773k | 2,219k | -41% |
| corpus `to_string` of the decoded cards | 8,726k | 8,479k | flat |

Instructions: corpus `parse` 5,267k to 4,082k, `decode` 13,817k to 8,273k, `bytes` 6,989k to 5,019k. Failed store forwards (`ls_bad_status2.stli_other:u`) 78.1k to 19.8k a `parse` pass, mispredicted branches 13.2k to 2.7k.

The machine ran another session's fuzzers at a load of 12 to 24 all along, so cycles read about 1.5 times the handoff's quiet baseline (1,551k for `parse`); instruction counts matched it, and only interleaved medians were compared. Criterion under that load (benches/README.md): `VcardCst::parse` 3.43 to 2.32 µs, parse + `decode` 9.25 to 6.64 µs, `decode` 4.99 to 3.67 µs, the rivals within 6% between runs; vcard-rs went from 10% behind calcard to 22% ahead. The criterion `to_string` and round-trip rows were bimodal under load (one base run 4.73 µs, the next two 2.67 µs) and are not read.

## What landed, each kept for its cycles

- A folded line built once (corpus `parse` -27%, `decode` -11%, `bytes` -18%): with the value colon on the first physical line, outside quotes, the head borrows the input and only the value is joined, into the buffer the value node keeps (`VcardValueNode::parse_cow`). It used to join the whole line, parse it borrowed, then copy every leaf again through `into_static`. The tokeniser's wire shape needs no `into_static` either. Without that colon the old path stays.
- The joined value sized first (`parse` -3.7%): one scan of the continuations reserves the value buffer and the fold list exactly, so a 40 KB `PHOTO` is copied once rather than through every doubling. A first variant reserving after the first line's copy measured -1.9% with 7% more instructions and was replaced.
- Lines filled in place (`parse` -4%): `VcardLine::take_into` fills a placeholder pushed onto the card's line list, where `take` returned a 160-byte line by value through two `Result`s. `VcardLine::take` keeps its signature over it. The dangling `=` of a quoted-printable value is inserted into the tokeniser's ordered shape (`insert_skipped_equals`, after every piece at or before its offset, which is what the stable sort of `prepend` did).
- The parameters in one walk (`parse` -4.5%): `VcardParamNode::take` finds the `;` ending a parameter, its first `=` and the `,` between its values in one pass, the `,` parity counted from the `=` as `split_param_values` did. `VcardParamNode::parse` keeps its behaviour and builds its leaves in one vector rather than two (flat on its own, smaller).
- Static line endings: `\r\n`, `\n` or nothing, borrowed, where a folded line allocated a `String` for its ending.
- The card's line list starts at 16 slots (`parse` -4.5%, card `parse` -9.5%): a card of fewer than 9 lines now holds up to 2.5 KB of spare slots until it is dropped, where the doubling from 4 held at most half of the list spare.
- The escaping mode stamped only when it is not the default the parser leaves on every node (card `parse` -2%), the two copies of that loop merged into `stamp_escaper`.
- Decoding validates UTF-8 strictly first and goes lossy only on failure, keeping an owned buffer rather than copying it, and the property, parameter and value kinds are matched exactly on an upper-cased stack copy (`from_name`) with no `String` built for an unknown name, which `decode` built and dropped for every `X-` property, twice (corpus `decode` -16%, instructions -25%).
- One walk over the components (`decode` -3.8%, then -7.7%, card `decode` -9.8%): the splitter is an iterator, so `component_at` stops at its component, and `N`, `ADR` (18 components), `ORG` and the unknown value read every component in one pass where each component walked the value from the start again.
- The decoded properties written in place (`decode` -3%, card `decode` -5.5%): the runs between `VERSION` lines go through an exact-length `extend`, where the filtered `collect` copied each 500-byte `VcardProp` twice.
- `to_bytes` lays the wire shape out inside its output (`bytes` -11%), writing past the logical line then moving it down, where each folded line allocated a buffer for its logical bytes; and sizes its output from each line's sealed length and pieces (`bytes` -8.4%, card `bytes` -17%).

## Tried and dropped

- `Vec::resize_with` for the placeholder line, which mime-rs measured writing in place: `parse` +2%.
- `from_name` upper-casing in a byte loop rather than `copy_from_slice` then `make_ascii_uppercase`: corpus `decode` +1.4%, card `decode` +4.8%, though 1% fewer instructions.
- `#[inline(always)]` on `decode_value` and `decode_value_kind`, to build the value in its property: flat (+0.3%, +1.2%).

## Verification

A differential crate against a copy of the baseline (`vcard-old`): the corpus and 600 rounds of random edits of it (87,600 inputs, 84,730 cards), through `parse` and `parse_many`, comparing the trees and errors (`Debug`), `to_bytes`, `to_string`, `decode` and its `to_string`, every cursor read on every line, then the same after setting component 2 of every line, which puts each value on the split path. A planted break in the old copy's parameter splitter failed it on the first corpus file. The suite under default, all and no features; clippy `--all-targets -D warnings` under all and no features; docs with `-Dwarnings`; `cargo build --no-default-features` depends on nothing; `cargo fmt`. The parse fuzz target ran 600 seconds from the corpus seed (376,833 runs), no crash.

The merge fuzz target found a merged card that does not reparse (`NonUtf8Header` on an `ADR;GEO="geo:..."` head running into a base64 blob), fuzz/artifacts/merge/crash-8b19b70e3f26662eb1f5db06655ac84eb0cf0614. The baseline tree fails on it the same way, so it predates this pass; it is left for its own change.

Next pass: [the-structure-scanned-once](../changes/the-structure-scanned-once/proposal.md).

Capabilities moved: none (parsing, decoding and serializing give the same bytes and values as before).
