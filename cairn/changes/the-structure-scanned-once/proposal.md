---
cairn: change
id: the-structure-scanned-once
status: active
created: 2026-11-02
---

# The structure scanned once

The handoff for the next speed pass over the parser, the decoded model and the serializers, rewritten at the end of [the-card-written-straight](../archive/the-card-written-straight/proposal.md) (2026-11-03) so a fresh session can take the lead on its own. Rewrite it at the end of each pass that takes it up, as mime-rs does with its cairn/changes/the-scratch-area/proposal.md: where things stand, where the time goes, the levers left, what was tried and dropped, the traps. Each pass lands under its own change id and log entry; this one stays active.

## Read first

- This repo's AGENTS.md, cairn/spec/parsing.md and cairn/spec/decoded-model.md: round-trip fidelity, the serialization fixpoint and line normalisation are the requirements a speed change must not move.
- cairn/log/2026-11-02-the-lines-built-in-place.md and cairn/log/2026-11-03-the-card-written-straight.md: what the two passes changed, each change with its cycles, and what they dropped.
- ~/code/pimalaya/mime/cairn: its passes (log/2026-11-02-the-tree-built-in-place.md, the-steps-inlined, the-state-carries-no-data) and changes/the-scratch-area/proposal.md, whose method and traps apply here.
- cairn/changes/an-edited-line-is-folded: a draft that will rewrite src/tree/wire.rs. A speed change there must keep its output byte for byte, and `lay_out_value` (the one-copy layout of an unsplit value) must follow whatever it changes in `lay_out`.

## Goal and rules

Make parsing, decoding and serializing faster with no behaviour change, in mime-rs's way:

- A change is kept only for its cycles, measured as below; a change that is flat goes, even if it looks cleaner to a profiler. Smaller source is a goal too.
- Every parser or model change gets the differential run below.
- No new dependency for the core. `cargo build --no-default-features` must stay dependency-free; the `parser` feature may trade memchr for simd-cpu (lever 5), nothing else.
- The crate's one `unsafe` is `tree::codec::unescape::text` (an ASCII proof standing in for `str::from_utf8`), which the user approved on 2026-11-03; any other `unsafe` needs a new yes, and every `unsafe` runs under Miri before landing.
- An API break needs the user's yes first (the boxed address had one on 2026-11-03).
- Each pass ends with: benches/README.md numbers, CHANGELOG, a log entry under the pass's own change id, this proposal rewritten as the next handoff, and a commit `perf: <what>` only once the user says yes to "commit now?" (no git write before that, no AI trailer in the message).

## Where things stand

After the second pass, on a quiet machine (load under 4), cycles a pass: corpus `parse` 746k, `decode` (parse then `decode`) 2,190k, `bytes` (parse then `to_bytes`) 992k, `to_string` of the decoded cards 326k; the short card parsed 300 times 856k, 2,568k, 1,078k and 519k. Decoding cards parsed beforehand takes 949k (corpus) and 1,416k (card), writing them with `to_bytes` 213k and 220k. Instructions a corpus pass: `parse` 3,494k, `decode` 6,847k, `bytes` 4,371k, `to_string` 1,317k. Criterion (benches/README.md): `VcardCst::parse` 594 ns against vparser 283 and ical_vcard 2,077; parse + `decode` 1,807 ns against calcard 2,838 and vcard_parser 58.1 µs; `decode` 1,018 ns, `to_bytes` 142 ns, `to_string` 363 ns. The first pass's figures were taken under a load of 12 to 24 and read about 1.5 times these in cycles and 2.5 times in criterion: compare within one state only.

Allocations a corpus pass: `parse` 2,129, `decode` 3,732, `to_bytes` 153, `to_string` 329; the short card 13, 27, 1 and 4. A decoded value is 120 bytes and a property 192 since the address is boxed; a parsed line is 160.

The corpus is tests/corpus: 146 `.vcf` files, 437 KB, 5,876 physical lines, of which 3,537 are fold continuations; three folded `PHOTO`s hold 41% of the bytes, logical lines run a median of 25 bytes, and 96 of the 2,246 content lines are `ADR`. Look at both the corpus and the short card before keeping a change: the passes' wins split very differently between them.

## Where the time goes

`perf record -c 200000 -e cycles:u` on the harness after this pass:

- `parse` (corpus): `VcardLine::take_into` 26%, `VcardLine::build` 19%, memmove 11%, memchr's `find_sse2` 9% (one call a physical line), `_int_malloc` 4%, the parameter lists' drop 4%.
- `parse` (card): `take_into` 32%, `build` 32%, memmove 8%, the parameter lists' drop 3.5%, `finish_grow` 3%, `take_card` 3%.
- `dec`, decode alone (corpus): `VcardLine::decode` 10%, `decode_items` 9%, memmove 7.5%, `_int_malloc` 7%, `malloc_consolidate` 7%, `VcardPropKind::from_name` 4.7%, `VcardParamNode::list` 4.4%, the decoded parameter lists' drop 4%, `find_sse2` 4%.
- `dec` (card): `VcardLine::decode` 16.5%, `decode_items` 16.5%, `malloc_consolidate` 9%, memmove 8%, `VcardParamNode::list` 6%, `memcmp` 5.4% (the byte-string matches of the kind lookups), `from_name` 4%.
- `encode`, `to_string` of decoded cards (corpus): memmove 16%, `escaped_at` 14%, `Display::fmt` 13%, `escape_into` 8%, `VcardValue::emit` 7.7%, `VcardValueOut::component` 5%, `_int_malloc` 4%, `memchr2` 4% (the 2.1 escapes), `escape_param` 4%.
- `encode` (card): memmove 15%, `escaped_at` 13.5%, `escape_param` 11%, `emit` 9%, `Display::fmt` 8.6%, `escape_into` 7.9%, `component` 11.5%, `VcardPropName::deref` 5.3%.
- `tob`, `to_bytes` alone (corpus): memmove 43%, `VcardLine::write_bytes` 27%, `VcardWirePart::write_bytes` 10%, `VcardParamNode::write_bytes` 9%. Most memmove calls copy a few bytes (a name, a parameter, a line ending): a call each.

What dominates the parser and the decoder now is not work but waiting: a multi-word value (a `Vec`, a `Cow`, the wire shape, a returned `Result`) written by 8-byte stores, then moved by 16-byte loads before the stores land, which store forwarding cannot serve. On the short card 78% of `take_into`'s samples wait on copying the freshly built wire shape into `build`'s argument, and most of `build`'s on moving the parameter value lists and the value `Cow`. Removing one such copy has every time moved the wait to the next (see Tried and dropped): the cure is structural, fewer values built in one place and moved to another.

## Levers, in order

1. Fewer moves between the tokeniser and the line. A line is built from pieces each returned through memory and moved into place: the wire shape, the value `Cow`, every parameter node and its value list. Every local cut was flat because the wait moved on; try a structural one, for instance the tokeniser recording a line as plain spans (offsets into the input, a few `u32`s held in registers) and one function writing the public `VcardLine` from them, field by field, with no intermediate value larger than two words. Measure the short card first: it is where the waits dominate.
2. Decode alone, which lost 1 to 5% to the boxed address though it gained 12 to 18% over the pass (the two memcpy calls a line went, and the 120-byte value's inline copies now wait on the callee's stores). The same cure applies: a line decoded into its property's slot (`decode_into`), the value written where it lands, the property list grown by a constant placeholder as the parser grows its lines. Parse plus decode, the headline row, gained from the box; keep it whole.
3. The kind lookups. `VcardPropKind::from_name` copies the name into a stack buffer with a variable-length `memcpy` call, upper-cases it byte by byte and matches byte strings, which calls `memcmp`; with `VcardParamKind::from_name` and `VcardValueKind::from_name` they take about 9.5% of card decode alone. A byte-loop upper-casing measured worse in the first pass; try matching on the length and the first byte, then comparing case-insensitively against one or two candidates, with no copy.
4. Small copies in `to_bytes` and `to_string`: memmove is 43% of `to_bytes` alone and 15% of `to_string`, mostly calls for a few bytes (a name, a parameter, `\r\n`). Writing the line ending and the fold pieces as pushes of constant bytes, and short pieces through a fixed-size copy, may spare the calls.
5. A structural scanner on simd-cpu, gated (the old lever 1). memchr is built without its `std` feature, so it never dispatches to AVX2 and each call pays its setup on 25-byte lines; but an inlined 16-byte scan of a line's first 64 bytes was flat, so the per-call setup is not the cost it looked. Only a kernel bench on the corpus's line lengths that beats memchr's per-call scans earns the scanner: one SIMD pass over a card classifying `\n`, `:`, `;`, `"`, `\\` and `,` into bit masks (AVX-512, AVX2, NEON and portable backends from `SimdCpu::current()`), read by the line splitter and the head and value splits. If the parser must ask for positions, it asks through an I/O-free coroutine, never a trait (the user's rule for pluggable behaviour). The codecs' traps apply: see the base64-codec and quoted-printable-codec cairns (the-gathered-blocks, the-column-chain).
6. Allocations: one parameter list a line with parameters and one value list a parameter (the public `Vec` fields fix their type), the decoded parameter and component lists, and the decoded property list (3 KB a card, past glibc's per-thread cache, so each card takes the large-request path and its `malloc_consolidate`, 7 to 9% of decode alone).

## The bench rows owed

Grow benches/parse.rs, so each lever shows on short and long lines in criterion too (owed since the first pass):

- The corpus through `parse_many`, for vcard-rs and for every rival that accepts each file. Whether the rivals parse all 146 files is unchecked: run each over the corpus first and state in the row how many cards it took.
- A 3.0 card with a folded 40 KB `PHOTO` (copy one from the corpus), and a 2.1 card in quoted-printable (`CHARSET`, soft breaks).
- The decoders against the crates they replaced: `decode_base64` against base64 0.23's `STANDARD` (on input it accepts, no whitespace) and `quoted_printable()` against quoted_printable 0.5 `Robust` (on ASCII input), both added as dev-dependencies only, with the token installed by `SimdCpu::install_detected()` at the start of the group (criterion owns `main`).

## Tried and dropped

From the first pass:

- `Vec::resize_with` for the placeholder line the card parser fills: `parse` +2% against `push`.
- `from_name` upper-casing in a byte loop rather than `copy_from_slice` then `make_ascii_uppercase`: corpus `decode` +1.4%, card `decode` +4.8%, though with 1% fewer instructions.
- `#[inline(always)]` on `decode_value` and `decode_value_kind`: flat.
- Reserving the folded value after copying the first physical line (a reallocation): -1.9% with 7% more instructions; sizing before the first copy kept instead (-3.7%).

From the second:

- `str::from_utf8` behind an `#[inline(never)]` helper returning `Option<&str>` in registers: decode alone +1.5%, the wait moved to the next copy of the returned `Cow`.
- `#[inline(always)]` on `VcardLine::decode`, the 504-byte property then built in its slot: +0.8% and +1.6%.
- `#[inline(always)]` on `build`: card parse +6.3% with 2.8% fewer instructions.
- The parameters pushed first and filled where they land: +0.8%.
- The wire shape recorded straight into the slot, not copied into `build`: parse -0.7%, card +0.9%.
- An inlined 16-byte scan of a physical line's first 64 bytes before memchr: parse +0.8%, card -1.1%, 3.1% more instructions.
- With the box: `#[inline(always)]` on `decode_value` and `decode_value_kind` (decode alone +6.8%); the value decoded before the parameters (+7.4%).

## Traps

- One binary measured twice in single runs can move 5%: only medians of interleaved runs decide. Two builds of near-identical code still differ by up to 1.5% from layout; an unaligned against an aligned build moved `decode` 10%.
- Another process on CPU 20, core 4's SMT sibling, makes a row bimodal (the card's parse and decode read 2,920k or 4,200k to 4,900k cycles from run to run, both binaries alike): `ab.sh` prints the minimum beside the median, interference only ever adding cycles. Check `uptime` and `ps` before a measuring run, and never run one while fuzzers or the differential run.
- Criterion under load misleads, even interleaved: on 2026-11-02 at a load of 24 its `to_string` and round-trip rows were bimodal. Read criterion for the rival ratios; decide with the cycle harness.
- A failed-forward site removed often moves the wait to the next load (five times in the second pass). Keep a change only for its cycles, not for a lower stall count or fewer instructions; fewer instructions can still be slower.
- `#[inline]` hints were ignored in mime-rs where `#[inline(always)]` moved cycles 4%, sometimes the wrong way: measure each.
- A block-scan predicate must join `==` tests with `|`: `matches!` compiles to a range check and a branch per byte, and LLVM does not vectorize it. A `u32` mask over 32-byte blocks compiled to four-byte loads; 16-byte blocks with a `u16` mask compile to `pcmpeqb`, `por`, `pmovmskb`.
- perf is user space only here (`perf_event_paranoid` 2): `cycles:u`, `instructions:u`, `ls_bad_status2.stli_other:u`, `ex_ret_brn_misp:u` work, call graphs fail, but branch stacks work (`perf record -b`, then `perf report -b --sort symbol_from,symbol_to`) and name a function's callers, memmove's among them. Sample at `-c 200000`: at `-c 10000` the kernel throttles and the counts collapse. callgrind (`nix-shell -p valgrind`, then `callgrind_annotate --tree=caller`) names the callers of `memcpy` and `malloc` exactly (but has no AVX-512).
- The cursor reads in the differential run only see unsplit values unless a line is edited first; the run below edits component 2 of every line for that.
- A script that copies the binary after `cargo build` must stop on a failed build, or it measures a stale binary; piping it into `tail` hides its exit status, so check its `built` line first (a failed build once let the differential run a stale binary).
- Edit code with the Edit tool, not `perl` or `sed` substitutions (a `|` delimiter turns `\|` into an alternation; braces in a replacement end a `s{}{}`). Never recover a file with git: stop and report.
- Never edit src/ while a background build, Miri or fuzz run may still compile it. Run fuzzers, Miri and the differential run away from the measuring core (`taskset -c 5-15`).
- The merge fuzz target crashes on fuzz/artifacts/merge/crash-8b19b70e3f26662eb1f5db06655ac84eb0cf0614 (a merged card that does not reparse), on the baseline too: a merge bug for its own change, not a speed regression. Run it with `-fork=3 -ignore_crashes=1` and check any new artifact against the baseline.
- `cargo doc` with `RUSTDOCFLAGS='--cfg docsrs'` fails on stable (E0554); check docs with `RUSTDOCFLAGS=-Dwarnings` alone. A public module's `//!` header may not link a `pub(crate)` item.

## How to measure

Keep the harness outside the repo, in a scratchpad, its path dependency pointing at the working tree. Cargo.toml:

```toml
[package]
name = "vcyc"
version = "0.0.0"
edition = "2024"

[dependencies]
vcard = { package = "vcard-rs", path = "/home/soywod/code/pimalaya/vcard" }
simd-cpu = { version = "0.0.1", git = "https://github.com/pimalaya/simd-cpu", rev = "f2eb72cff2fafad8ae135e00685bc19f9e80e5cc", features = ["std"] }

[profile.release]
codegen-units = 1
debug = 1

[workspace]
```

src/main.rs, run as `vcyc <work> <passes>`, `<work>` one of `parse`, `decode` (parse then `decode`), `bytes` (parse then `to_bytes`), `dec` (`decode` of cards parsed beforehand), `tob` (`to_bytes` of cards parsed beforehand), `encode` (`to_string` of cards decoded beforehand), or any of them prefixed `card-` for the criterion card parsed 300 times a pass:

```rust
use std::{fs, hint::black_box, path::Path};
use vcard::tree::cst::VcardCst;

fn walk(p: &Path, out: &mut Vec<Vec<u8>>) {
    let mut es: Vec<_> = fs::read_dir(p).unwrap().flatten().map(|e| e.path()).collect();
    es.sort();
    for e in es {
        if e.is_dir() { walk(&e, out) } else if e.extension().is_some_and(|x| x == "vcf") { out.push(fs::read(e).unwrap()) }
    }
}

fn main() {
    let _ = simd_cpu::SimdCpu::install_detected();
    let mut a = std::env::args().skip(1);
    let work = a.next().unwrap();
    let n: usize = a.next().unwrap().parse().unwrap();
    let mut corpus = Vec::new();
    walk(Path::new("/home/soywod/code/pimalaya/vcard/tests/corpus"), &mut corpus);
    if let Some(short) = work.strip_prefix("card-") {
        corpus = vec![CARD.as_bytes().to_vec(); 300];
        return run(short, n, &corpus);
    }
    run(&work, n, &corpus);
}

fn run(work: &str, n: usize, corpus: &[Vec<u8>]) {
    let csts = || corpus.iter().flat_map(|f| VcardCst::parse_many(f.as_slice()).flatten()).collect::<Vec<_>>();
    if work == "dec" || work == "tob" {
        let csts = csts();
        for _ in 0..n {
            for cst in &csts {
                if work == "dec" { black_box(black_box(cst).decode()); } else { black_box(black_box(cst).to_bytes()); }
            }
        }
        return;
    }
    if work == "encode" {
        let csts = csts();
        let cards: Vec<_> = csts.iter().map(|c| c.decode()).collect();
        for _ in 0..n {
            for card in &cards {
                black_box(black_box(card).to_string());
            }
        }
        return;
    }
    for _ in 0..n {
        for f in corpus {
            for card in VcardCst::parse_many(black_box(f.as_slice())) {
                let Ok(card) = card else { continue };
                match work {
                    "parse" => { black_box(&card); }
                    "decode" => { black_box(card.decode()); }
                    "bytes" => { black_box(card.to_bytes()); }
                    _ => panic!(),
                }
            }
        }
    }
}

// NOTE: the CARD constant of benches/parse.rs, copied verbatim.
const CARD: &str = "...";
```

Build it aligned, each version in its own target directory, and copy the binary aside under its own name, stopping on a failed build:

```sh
RUSTFLAGS='-C llvm-args=-align-all-functions=6 -C llvm-args=-align-loops=64' \
  nix-shell /home/soywod/code/pimalaya/vcard/shell.nix \
  --run "CARGO_TARGET_DIR=$PWD/target-aligned cargo build --release -q" \
  && cp target-aligned/release/vcyc bin/vcyc-new && echo built
```

Count cycles at two pass counts and take the difference, which removes the corpus loading; compare two versions by interleaving 7 runs of each (11 for a close call) and comparing medians and minimums, the instruction count beside (`nix-shell -p perf --run "./ab.sh vcyc-old vcyc-new 7 parse decode bytes encode card-parse card-decode card-bytes card-encode"`, from bin/):

```sh
A=$1; B=$2; r=$3; shift 3
one() {
  for n in 20 120; do
    taskset -c 4 perf stat -x, -e cycles:u,instructions:u ./$1 $2 $n 2>&1 >/dev/null | cut -d, -f1 | tr '\n' ' '
    echo
  done | awk 'NR==1{c=$1;i=$2} NR==2{printf "%.1f %.1f\n", ($1-c)/100e3, ($2-i)/100e3}'
}
for w in "$@"; do
  : > a.$w.txt; : > b.$w.txt
  for i in $(seq $r); do one $A $w >> a.$w.txt; one $B $w >> b.$w.txt; done
  ma=$(sort -n a.$w.txt | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
  mb=$(sort -n b.$w.txt | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
  na=$(sort -n a.$w.txt | head -1 | cut -d' ' -f1); nb=$(sort -n b.$w.txt | head -1 | cut -d' ' -f1)
  ia=$(head -1 a.$w.txt | cut -d' ' -f2); ib=$(head -1 b.$w.txt | cut -d' ' -f2)
  awk -v w=$w -v ma=$ma -v mb=$mb -v na=$na -v nb=$nb -v ia=$ia -v ib=$ib 'BEGIN{printf "%-12s med %8.1f %8.1f %+6.1f%%  min %8.1f %8.1f %+6.1f%%  instr %8.1f %8.1f %+6.1f%%\n", w, ma, mb, (mb-ma)/ma*100, na, nb, (nb-na)/na*100, ia, ib, (ib-ia)/ia*100}'
done
```

To count allocations, a second harness wraps `System` in a counting `#[global_allocator]` (allocations, reallocations, bytes and a power-of-two size histogram, counted only inside a closure) and reports `parse`, `decode`, `to_bytes` and `to_string` for one corpus pass and one card. After the second pass: corpus parse 2,129 allocations, decode 3,732, `to_bytes` 153, `to_string` 329; the card 13, 27, 1 and 4.

To name stalling copies, sample the failed forwards on the debug-info build and annotate the symbol as `perf report` prints it, and name a copy's callers from branch stacks:

```sh
nix-shell -p perf --run "taskset -c 4 perf record -c 2000 -e ls_bad_status2.stli_other:u ./vcyc-new parse 300 \
  && perf annotate --stdio -s '<vcard::tree::line::VcardLine>::take_into'"
nix-shell -p perf --run "taskset -c 4 perf record -c 50000 -e cycles:u -b ./vcyc-new card-dec 100 \
  && perf report -b --sort symbol_from,symbol_to --stdio | grep memmove"
```

For a baseline, copy the tree before the pass (src, benches, examples, Cargo.toml, Cargo.lock) into the scratchpad and point a second harness at it. For criterion, build the baseline copy's bench and the working tree's in their own target directories (`cargo bench --bench parse --no-run`), run the two executables alternately with `--bench --output-format bencher` on core 4, three times each, and take the medians.

## The differential run

For any parser or model change: copy the previous src/ and Cargo.toml into a scratch crate renamed `vcard-old` (lib name `vcard_old`, its `[features]` and `[dependencies]` kept, the `[[bench]]`, `[[example]]` and `[dev-dependencies]` sections dropped), then for the corpus and 300 to 600 rounds of random edits of it (a byte removed, replaced or inserted from an alphabet of CR, LF, space, tab, `:`, `;`, `,`, `"`, `=`, `\`, `^`, `ENCODING=QUOTED-PRINTABLE`, `CHARSET=`, `BEGIN:VCARD`, `END:VCARD`, `VERSION:2.1`, `VERSION:3.0`, `VERSION:4.0`, `item1.`, non-ASCII bytes, `\r\n `, `\r\n\t`, `=\r\n`, the escapes `\n`, `\N`, `\,`, `\;`, `\\`, the carets `^n`, `^'`, `^^`, `;VALUE=uri`, `;TYPE="a,b"`, whole `ADR`, `N`, `GEO`, `GENDER` and `X-` lines, and chunks copied from elsewhere in the file; files over 4 KB cut to a random 4 KB window), through `parse` and `parse_many`, assert:

- `format!("{:?}")` of both results match, trees and errors;
- `to_bytes` and `to_string` of both match;
- `decode` of both match (`Debug`), and for the decoded cards: `to_string`, `encode` (`Debug`, `to_bytes`, and its `to_string` against the direct `to_string`), and every property's `encode` under each of the three escapers;
- the same on the decoded model salted under each version: a salt (separators, a backslash, a line break, carets, a quote, non-ASCII, `\r\n`) put at the front, the back or both ends of every string a model can hold, a group set on every fourth property and empty lists filled, through a walker written once as a macro for both crates;
- every cursor read (`text`, `bytes`, `list`, `component` 0, 1, 2 and 7, `quoted_printable`, `charset`) on every line matches;
- then, after `set_component(2, &["x,y", "z;"])` on every line of both, `to_bytes`, `to_string`, `decode`, the model checks and every cursor read match again.

Save a failing input before panicking. Plant a break in the old copy, in the code the change touches, and see the run fail before trusting it.

## Before landing a pass

`cargo test` under default, all and no features; clippy `--all-targets -- -D warnings` under all and no features; `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features`; `cargo build --no-default-features` depending on nothing; the unit tests and the corpus, codecs and encoding tests under Miri when `unsafe` changed (`nix-shell ~/code/pimalaya/quoted-printable-codec/fuzz/shell.nix --run "cargo miri test --lib"` from this repo, its nightly carrying Miri); the fuzz targets for ten minutes each (`nix-shell fuzz/shell.nix --run "cd fuzz && cargo fuzz run -O parse -- -max_total_time=600"` on `taskset -c 5-7`, merge forked as above on `8-11`), seeded from a copy of tests/corpus in fuzz/corpus/parse and fuzz/corpus/merge; `cargo fmt`; the differential run; the cycle harness on every workload against the pass's baseline; a criterion run interleaved with the baseline's; benches/README.md numbers; CHANGELOG; a log entry; this proposal rewritten; then ask the user before committing.
