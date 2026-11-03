---
cairn: log
change: the-card-written-straight
landed: 2026-11-03
---

# The card written straight

The second speed pass over the parser, the decoded model and the serializers, taken from the handoff [the-structure-scanned-once](../changes/the-structure-scanned-once/proposal.md), which stays active, rewritten for the next pass. The baseline is the tree as committed after [the-lines-built-in-place](./2026-11-02-the-lines-built-in-place.md) (ec5f0c1). Two of its changes needed the yes the handoff reserves to the user, boxing the address (lever 5) and an `unsafe` UTF-8 shortcut (lever 6); both were given on 2026-11-03.

## Results

User-space cycles a pass (`perf stat -e cycles:u` at 20 and 120 passes, the difference taken, medians of 11 interleaved runs of each binary, both built with `-align-all-functions=6 -align-loops=64`, `taskset -c 4`, on a quiet machine), over tests/corpus and over the criterion card parsed 300 times:

| workload | before | after | delta |
| --- | ---: | ---: | ---: |
| corpus `parse` | 853k | 746k | -12.5% |
| corpus `decode` (parse then `decode`) | 2,736k | 2,190k | -20.0% |
| corpus `bytes` (parse then `to_bytes`) | 1,162k | 992k | -14.6% |
| corpus `to_string` of the decoded cards | 5,719k | 326k | -94.3% |
| card `parse` | 994k | 856k | -13.8% |
| card `decode` | 2,953k | 2,568k | -13.0% |
| card `bytes` | 1,229k | 1,078k | -12.3% |
| card `to_string` | 4,958k | 519k | -89.5% |
| corpus `decode` of cards parsed beforehand | 1,156k | 949k | -17.8% |
| card `decode` of cards parsed beforehand | 1,611k | 1,416k | -12.1% |
| corpus `to_bytes` of cards parsed beforehand | 244k | 213k | -12.7% |
| card `to_bytes` of cards parsed beforehand | 241k | 220k | -8.6% |

Instructions: corpus `parse` 4,081k to 3,494k, `decode` 8,274k to 6,847k, `bytes` 5,018k to 4,371k, `to_string` 14,273k to 1,317k. Allocations a corpus pass (a counting allocator): `to_string` 15,686 to 329, the card's 124 to 4; `decode` 3,636 to 3,732 (one box an `ADR` line) with the bytes allocated down from 1.25 MB to 0.69 MB.

Criterion, medians of three interleaved runs (benches/README.md): `VcardCst::parse` 749 to 594 ns, parse + `decode` 2,216 to 1,807 ns, `decode` 1,184 to 1,018 ns, `to_bytes` 156 to 142 ns, the round trip 935 to 782 ns, `to_string` 3,569 to 363 ns; the rivals moved within 4%. vparser stays 2.1 times faster at the content-line level, and calcard is now 57% slower than parse + `decode`.

## What landed, each kept for its cycles

Each figure is the change against the build before it, interleaved medians on core 4.

- The decoded card written straight to text: `Display` for `Vcard` encoded the card into a syntax tree (a leaf allocation per value, about seven allocations a line) and printed the tree, checking every leaf for UTF-8 again. It now writes the text straight from the model. Each value type states its wire shape once, in an `emit` writing one verbatim value or a run of components into a `VcardValueOut`, which `VcardCodec::encode` points at a node and `Display` at text, so the two cannot drift apart; the parameter's name and values come from one `wire` match that `encode` reads too. Corpus `to_string` -56%, short card -84%.
- The value escapes found sixteen bytes at a time: `escape_with` tested every byte with a branchy `matches!`. A 16-byte block's hits gathered into a bit mask compile to four `pcmpeqb`, three `por` and a `pmovmskb`, the position given by `trailing_zeros`; the predicate must join `==` tests with `|`, a `matches!` compiling to a branch per byte. `escape_with` and the new `escape_into` share it, and 2.1 uses `memchr2`. Corpus `to_string` a further -83%, card -10%.
- The text sized up front, 64 bytes plus 32 a property: corpus -21%, card -24% (a 64 KB buffer, the upper bound, gave -23% and -26%). A long value grows it once, by its own length.
- The parameter escapes tested on bytes rather than `char` patterns: -2.8% and -2.6%.
- A value checked for UTF-8 once: every comma-separated piece paid its own `from_utf8` and its own backslash search. The components now come as slices of the value's text, checked once (`Component::Text`), and are unescaped by `unescape_text`, which never checks again: every escape is ASCII. Decode alone -7.6% (card) and -9.6% (corpus).
- A value with no backslash cut at every separator (`str::split`, borrowing every item), found by one search for a backslash a value: card decode alone -3.1%, parse and decode -2.0%; the corpus row moved +1.6% (11 runs) with 1.9% fewer instructions, within the layout spread the handoff warns of, and was kept for the card.
- One split of the group from the name in `decode`, where `group()` and `bare_name()` each searched for the dot: -0.5% to -1.7%, instructions -2%.
- A constant placeholder line: `push(VcardLine::empty())` built the 160-byte line on the stack and copied it out, the copy waiting on the stores; `const { VcardLine::empty() }` is copied from read-only data. Parse -2.5%, card -4.7%.
- The placeholder's fields overwritten without a drop (`mem::forget(mem::replace(..))`, the empty line holding no allocation): parse -5.1%, card -7.4%, instructions -4% and -7%.
- The folds read once: the sizing scan records each fold, and the copy takes each continuation's length from the offset of the fold after it, where every continuation was searched for its line end twice. A whitespace-only first line keeps the old loop, since a continuation may still be stripped there. Parse -4.7%, bytes -3.2%, decode -3.0%, instructions -6%; the card has no folds.
- A folded value laid out once: `to_bytes` wrote a line's logical bytes, copied them past themselves with the wire pieces and moved them down, three copies of a 40 KB `PHOTO`. The head is written first and the value read straight from its node (`lay_out_value`); pieces at offset 0 are rotated in front of the head, and a piece inside the head takes the old path. `to_bytes` alone -13% (corpus), -10% (card); bytes -3.1% and -1.2%. A line with no pieces returns at once (without that, the card regressed 6.5% in instructions).
- `VcardValue::Adr(Box<VcardAdr>)` (lever 5): a value went from 432 to 120 bytes and a property from 504 to 192, so a card's property list shrinks from 7.5 to 3 KB. Parse and decode -2.3% (corpus) and -4.2% (card), `to_string` -4.9% and -11.8% (11 runs). Decode alone, from a card parsed beforehand, got slower, +5.1% and +1.2%: the two memcpy calls a line went, and the inline 16-byte copies of the smaller value now wait on the callee's stores where the calls did not. Kept for the headline rows and the memory.
- An ASCII proof for UTF-8 (lever 6): `text()` returns `str::from_utf8_unchecked` when `<[u8]>::is_ascii()` holds, inlined and answered in a register, and `str::from_utf8` otherwise, whose result comes back through memory and stalled the 16-byte copy of it. It reads each line's head and each value. Decode -9.6% and -4.5%, decode alone -9.1% and -5.1%, parse -2.5% (the card flat, instructions -6%). The crate's first `unsafe`, with a SAFETY comment, run under Miri (see Verification).

## Tried and dropped

- `str::from_utf8` behind an `#[inline(never)]` helper returning `Option<&str>` in two registers, so its result is read by two 8-byte loads that forward: decode alone +1.5%. `unescape_with` fell from 479 samples to 31 and the wait moved to the next copy of the 24-byte `Cow` it returns (memmove, `VcardLine::decode`, `Pieces::next`).
- `#[inline(always)]` on `VcardLine::decode`, so the 504-byte property is built in the list's slot: the copy went, the cycles did not (+0.8%, +1.6%).
- `#[inline(always)]` on `build`: card parse +6.3% with 2.8% fewer instructions.
- The parameters pushed first and filled where they land (`VcardParamNode::take_into`), so their value lists are never moved: +0.8%.
- The wire shape recorded straight into the slot, so `build` stops copying it (78% of `take_into`'s samples on the card wait on that copy): parse -0.7%, card +0.9%.
- The first 64 bytes of a physical line searched by the inlined block scan before memchr: parse +0.8%, card -1.1%, instructions +3.1%.
- With the box: `#[inline(always)]` on `decode_value` and `decode_value_kind` (decode alone +6.8%), and the value decoded before the parameters so its stores land before the move (decode alone +7.4%).
- 32-byte blocks with a `u32` mask in the escape scan: LLVM loads them four bytes at a time; 16-byte blocks kept (codegen read, not timed).

## Verification

A differential crate against a copy of the baseline (`vcard-old`): the corpus and 600 rounds of random edits of it (87,600 inputs, 77,937 cards), through `parse` and `parse_many`, comparing the trees and errors (`Debug`), `to_bytes`, `to_string`, `decode` and every cursor read on every line, then the same after setting component 2 of every line. It now also compares, for every decoded card, `to_string`, `encode` (`Debug`, `to_bytes`, and its `to_string` against the direct writer) and every property's `encode` under the three escapers, and does it again on the decoded model salted under each version: separators, backslashes, line breaks, carets, quotes, non-ASCII and `\r\n` put at the front, the back or both ends of every string, a group on every fourth property and empty lists filled. Planted breaks in the old copy (the 2.1 line-break escape written `\N`, the 3.0 `GEO` pair swapped) failed it on corpus files 1 and 52. The suite under default, all and no features; clippy `--all-targets -D warnings` under all and no features; docs with `-Dwarnings`; `cargo build --no-default-features` depends on nothing; `cargo fmt`. Miri: the 193 unit tests and the corpus, codecs, encoding, rfc, lens, coverage and GitHub tests. The parse fuzz target ran 600 seconds from its seeded corpus (937,178 runs) and the merge target 612 seconds forked three ways (329,758 runs), with no crash and no new artifact; the merge target's known crash (fuzz/artifacts/merge/crash-8b19b70e3f26662eb1f5db06655ac84eb0cf0614) did not recur in that run.

The machine ran another session's fuzzers at a load of 12 to 24 at the start, then went quiet (load 1 to 4); every figure in the tables was taken quiet. The baseline's corpus `parse` read 1,485k cycles under that load and 853k quiet, so cycles are comparable only within one state.

Next pass: [the-structure-scanned-once](../changes/the-structure-scanned-once/proposal.md), rewritten.

Capabilities moved: decoded-model (`VcardValue::Adr` holds its `VcardAdr` boxed); parsing, decoding and serializing give the same bytes and values as before.
