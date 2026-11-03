---
cairn: log
change: the-parts-written-in-place
landed: 2026-11-03
---

# The parts written in place

The third speed pass over the parser, the decoded model and the serializers, taken from the handoff [the-structure-scanned-once](../changes/the-structure-scanned-once/proposal.md), which stays active, rewritten for the next pass. The baseline is the tree as committed after [the-card-written-straight](./2026-11-03-the-card-written-straight.md) (df9565d). No behaviour change, no API change, and no new `unsafe`.

## Results

User-space cycles a pass (`perf stat -e cycles:u` at 20 and 120 passes, the difference taken, medians of 7 interleaved runs of each binary, all built with `-align-all-functions=6 -align-loops=64`, `taskset -c 4`), over tests/corpus and over the criterion card parsed 300 times. The baseline and the final tree were each built in four layouts (see Measuring) and the eight binaries interleaved; the table gives layout 0 and the mean delta over the four, every row gaining in every layout:

| workload | before | after | delta | four layouts |
| --- | ---: | ---: | ---: | ---: |
| corpus `VcardCst::parse` | 699k | 578k | -17.4% | -18.3% |
| corpus `parse` (`parse_many`) | 744k | 623k | -16.2% | -16.5% |
| corpus `decode` (parse then `decode`) | 2,202k | 2,061k | -6.4% | -6.2% |
| corpus `bytes` (parse then `to_bytes`) | 982k | 849k | -13.5% | -13.4% |
| corpus `to_string` of the decoded cards | 322k | 314k | -2.5% | -2.9% |
| card `VcardCst::parse` | 849k | 647k | -23.8% | -24.9% |
| card `parse` | 857k | 674k | -21.3% | -20.5% |
| card `decode` | 2,618k | 2,256k | -13.8% | -13.0% |
| card `bytes` | 1,071k | 878k | -18.0% | -17.5% |
| card `to_string` | 520k | 461k | -11.3% | -11.2% |
| corpus `decode` of cards parsed beforehand | 955k | 902k | -5.6% | -6.0% |
| card `decode` of cards parsed beforehand | 1,422k | 1,248k | -12.2% | -11.3% |
| corpus `to_bytes` of cards parsed beforehand | 212k | 199k | -6.0% | -6.5% |
| card `to_bytes` of cards parsed beforehand | 220k | 206k | -6.4% | -5.9% |

Instructions a corpus pass: `VcardCst::parse` 3,317k to 2,745k, `parse` 3,494k to 2,980k, `decode` 6,847k to 6,351k, `bytes` 4,371k to 3,933k, `to_string` 1,317k to 1,306k; decode alone 3,223k to 3,262k, its gain all waiting. Failed store forwards (`ls_bad_status2.stli_other:u`, 100 passes): card `parse` 6.03M to 1.05M (about 12 a line to 2), card decode alone 9.20M to 1.49M (20 a property to 3), corpus `parse` 2.29M to 0.67M, corpus decode alone 3.57M to 1.10M.

Allocations a corpus pass (a counting allocator): `parse` 2,129 and `decode` 3,732 as before, the decoded cards' bytes 688 KB to 575 KB (an item list is sized once rather than grown to four); `to_string` 329 to 309, the card's 4 to 2; `to_bytes` 153 as before. Decoding now makes 137 shrinking reallocations a corpus pass, the unescape slow path's `into_boxed_str`.

Criterion, medians of three interleaved runs (benches/README.md): `VcardCst::parse` 602 to 497 ns, parse + `decode` 1,893 to 1,554 ns, `decode` 1,042 to 916 ns, `to_bytes` 147 to 133 ns, the round trip 795 to 658 ns, `to_string` 376 to 344 ns; the rivals moved within 3.2%. vparser is now 1.7 times faster at the content-line level (2.1 before), and calcard is 88% slower than parse + `decode`.

## Measuring

- The second pass compared a baseline built from the scratchpad copy with a final tree built from the repository. Panic messages embed the source path, so the two had .rodata sections 384 bytes apart, which shifted all of .text by six cache lines, and in this pass's first comparison of identical code the `to_bytes` rows read 20 to 60% slower in the scratchpad build (front-end bound: `de_no_dispatch_per_slot.no_ops_from_frontend` 468M against 122M slots), every other row within 2%. A `-C metadata` seed changes symbol names, not the layout. The harness now remaps the baseline copy's path to the repository's (`--remap-path-prefix`), which gives the same section and function addresses for the same code, and builds four layouts by remapping the path to prefixes of four lengths; four builds of the same code agree within 2.6% on every row.
- From 08:47 to 09:46 the ical-rs pass also measured on CPU 4. The per-change screening figures below were mostly taken in that window: interleaved, so both binaries shared the interference, but noisier than the final table. The final table and the criterion run were taken after 09:51, with CPU 4 to themselves; an earlier final run, of the tree before its last change (the exact reservation, which touches decode alone), agrees with it within 2.2 points on every other row.

## What landed, each kept for its cycles

Each figure is the change against the build before it, layout 0, 7 interleaved runs.

- The common content line written into its slot (lever 1). A line that is one physical line, not blank, indented, folded or soft-broken, has a wire shape equal to its logical shape, so it records none: the name, the value, the line ending and the shape's sealed length are stored into the empty line straight from registers, and its parameters are pushed onto the line's list as constant empty nodes and filled where they land, their value lists grown in place (`VcardParamNode::take_into`). Every other line takes the old path (`take_wired`). Against the baseline: corpus parse -9.0%, card -14.9%, card bytes -14.4%, card decode -6.8%; the card's failed forwards 6.08M to 1.99M.
- The empty line grown by `resize_with`, its fields written by constant stores, where a pushed constant was a 160-byte `memcpy` call: -0.4% to -1.9% on every row.
- The `BEGIN` line filled where it stays and passed to `take_body` by reference, the escapers stamped before the `END` line is moved out, and `VcardCst::parse` no longer tokenising its first line twice (it took it, checked it was `BEGIN`, then took it again). Flat on `parse_many` (+1.8%, -0.6%); kept for `VcardCst::parse`, whose rows gain about 2 (corpus) and 4 (card) points more than `parse_many`'s in the final table.
- Properties decoded into their slots (lever 2). The card's property list grows by a constant empty property and each line is decoded into it field by field (`VcardLine::decode_into`); the value is written by one store per kind (`decode_value_kind_into`), not matched out of fifteen arms and moved; `unescape_text` and `unescape_param` are inlined with their slow paths out of line; item lists are sized once. Decode alone -5.0% (corpus) and -1.7% (card), failed forwards 8.76M to 4.65M.
- The kind lookups (lever 3). `VcardPropKind`, `VcardParamKind` and `VcardValueKind::from_name` copied the name into a stack buffer by a variable-length `memcpy`, upper-cased it and matched byte strings with `memcmp` calls, the copy's stores failing to forward to the compares. They now pick one candidate by the length and the first byte (the second for the few that share both) and compare it whole, case aside, never copying the name; a test checks every spelling, recased, one byte off, cut and grown, against a search of every spelling. With the byte splits below and the scalar values stored from registers: decode alone -1.2% (corpus) and -5.1% (card), parse + decode -3.4% and -2.2%.
- The splits by byte. Once the decode's copies went, the waits surfaced in `str::split(';')` and `split_once('.')`, whose char searcher confirms every `memchr` hit with a `bcmp` call (`memcmp` rose to 13% of cycles); `PlainPieces` cuts at an ASCII byte found by an inlined search.
- Each arm storing its own value. LLVM stages a `Cow` that two arms produce in a stack temporary and moves it with a 16-byte load, even for the arm that builds it in registers. The scalar kinds store a value with nothing to unescape straight into the slot (`VcardValueNode::plain`) and unescape out of line (`decode_text_into`); a parameter with a caret is decoded out of line (`decode_carets`); a list pushes each item from its own arm; the unescape slow paths return a `Box<str>`, in two registers.
- The structured values filled in place: `N` and `ADR` decode their components into their empty lists (`decode_component_lists_into`), an unknown value onto its component list, and `Components::next` is `#[inline(always)]`, its component returned in registers rather than copied out of memory: card decode alone -5.2% (corpus -0.6%), failed forwards 2.66M to 1.70M. `#[inline]` alone was ignored.
- The line splitter's block scans (part of lever 5). The line feed (`physical_line`), the value colon and the name's semicolon (`head_marks`, which settles a head with no quote before its colon and leaves any other to the byte loops) are found sixteen bytes at a time by the inlined scan the escapes use, run over the rest of the input so a short line still gets whole blocks, in place of a `memchr` call a line and two byte loops: parse -4.6% and -3.9%, `VcardCst::parse` -3.7% and -1.6%, bytes -4.5% (the card +0.2%), instructions -6% to -9%.
- Short pieces of `to_bytes` by fixed-size moves (lever 4). A name, a parameter, a value of sixteen bytes or fewer and the line ending were a `memcpy` call each; `put` writes them over sixteen zeroes by two overlapping moves and cuts the zeroes off, the buffer sized with sixteen bytes of headroom: `to_bytes` alone -6.4% and -6.3%, bytes -3.2% and -7.3%.
- A parameter value written in place by the text writer (`escape_param_into`), where `escape_param` returned a `Cow` through memory: card `to_string` -2.5% with 6% fewer instructions, the corpus flat (+0.9%); its row reads -11% in the final table.
- The decoded parameter list reserved exactly before it is extended: an empty list extended reserves room for four, `collect` had sized it exactly. Kept for the memory; the cycles were mixed (corpus decode rows 1 to 2.5% faster without it, the card's parse + decode 1.5 to 3% slower).

## Tried and dropped

- The `BEGIN` line passed to `take_body` by value, with `parse_bare` also growing by `resize_with`: parse +3.0%, card +3.9%, 543 more instructions a card; the line was copied right after its stores, and `resize_with` stopped inlining with two callers. Passed by reference instead, `parse_bare` back on `push`.
- A first in-place decode with the slow paths boxed, before the kind lookups and the splits: decode alone went from -5.5% to -0.3%, the waits moved into `from_name`'s `memcmp` and `str::split`'s `bcmp`.
- `#[inline]` on `Components::next`: LLVM kept the call, flat.
- `list` inlined, every value borrowed after a pre-pass for carets: card decode alone +2.7%, 1% more instructions.
- The card built in the iterator's item, `take_body` writing the rest through a reference and the trailing blank lines read first: flat to +2.5% (card parse).

## Verification

The differential crate against a copy of the baseline (`vcard-old`), unchanged from the second pass: the corpus (146 files, 299 cards) and 600 rounds of random edits of it (87,600 inputs, 77,937 cards), comparing the trees and errors, `to_bytes`, `to_string`, `decode`, `encode` and every property's encode under the three escapers on the decoded and the salted models, and every cursor read, before and after setting component 2 of every line; all agree. Planted breaks in the old copy (commas inside quotes split in the parameter tokeniser, `URL` unknown to the kind lookup) failed it on corpus files 2 and 10. The suite under default (310 tests), all (334) and no features (48), with two new tests (the kind lookups against a search of every spelling, failed by a planted dispatch error; `put` at every length from 0 to 26); clippy `--all-targets -D warnings` under all and no features; docs with `-Dwarnings`; `cargo build --no-default-features` depends on nothing and warns of nothing; `cargo fmt`. The parse fuzz target ran 601 seconds from its seeded corpus (854,720 runs) and the merge target 607 seconds forked three ways (196,885 runs), with no crash and no new artifact. The crate's one `unsafe` is unchanged, so Miri did not run.

Next pass: [the-structure-scanned-once](../changes/the-structure-scanned-once/proposal.md), rewritten.

Capabilities moved: none; parsing, decoding and serializing give the same bytes and values as before.
