---
cairn: change
id: the-structure-scanned-once
status: active
created: 2026-11-02
---

# The structure scanned once

The handoff for the next speed pass over the parser, the decoded model and the serializers, rewritten at the end of [the-parts-written-in-place](../archive/the-parts-written-in-place/proposal.md) (2026-11-03) so a fresh session can take the lead on its own. Rewrite it at the end of each pass that takes it up, as mime-rs does with its cairn/changes/the-scratch-area/proposal.md: where things stand, where the time goes, the levers left, what was tried and dropped, the traps. Each pass lands under its own change id and log entry; this one stays active.

## Read first

- This repo's AGENTS.md, cairn/spec/parsing.md and cairn/spec/decoded-model.md: round-trip fidelity, the serialization fixpoint and line normalisation are the requirements a speed change must not move.
- The three passes' logs, each change with its cycles and what was dropped: cairn/log/2026-11-02-the-lines-built-in-place.md, cairn/log/2026-11-03-the-card-written-straight.md, cairn/log/2026-11-03-the-parts-written-in-place.md.
- ~/code/pimalaya/mime/cairn: its passes (log/2026-11-02-the-tree-built-in-place.md, the-steps-inlined, the-state-carries-no-data) and changes/the-scratch-area/proposal.md, whose method and traps apply here.
- cairn/changes/an-edited-line-is-folded: a draft that will rewrite src/tree/wire.rs. A speed change there must keep its output byte for byte, and `lay_out_value` (the one-copy layout of an unsplit value) must follow whatever it changes in `lay_out`.

## Goal and rules

Make parsing, decoding and serializing faster with no behaviour change, in mime-rs's way:

- A change is kept only for its cycles, measured as below; a change that is flat goes, even if it looks cleaner to a profiler. Smaller source is a goal too.
- Every parser or model change gets the differential run below.
- No new dependency for the core. `cargo build --no-default-features` must stay dependency-free; the `parser` feature may trade memchr for simd-cpu (lever 5), nothing else.
- The crate's one `unsafe` is `tree::codec::unescape::text` (an ASCII proof standing in for `str::from_utf8`), which the user approved on 2026-11-03; any other `unsafe` needs a new yes, and every `unsafe` runs under Miri before landing.
- An API break needs the user's yes first.
- Each pass ends with: benches/README.md numbers, CHANGELOG, a log entry under the pass's own change id, this proposal rewritten as the next handoff, and a commit `perf: <what>` only once the user says yes to "commit now?" (no git write before that, no AI trailer in the message).

## Where things stand

After the third pass, on CPU 4 with nothing else on core 4, cycles a pass at layout 0 (medians of 7 interleaved runs): corpus `VcardCst::parse` 578k, `parse` (`parse_many`) 623k, `decode` (parse then `decode`) 2,061k, `bytes` (parse then `to_bytes`) 849k, `to_string` of the decoded cards 314k; the short card parsed 300 times 647k, 674k, 2,256k, 878k and 461k. Decoding cards parsed beforehand takes 902k (corpus) and 1,248k (card), writing them with `to_bytes` 199k and 206k. Instructions a corpus pass: `VcardCst::parse` 2,745k, `parse` 2,980k, `decode` 6,351k, `bytes` 3,933k, `to_string` 1,306k. Criterion (benches/README.md): `VcardCst::parse` 497 ns against vparser 290 and ical_vcard 2,116; parse + `decode` 1,554 ns against calcard 2,922 and vcard_parser 59.6 µs; `decode` 916 ns, `to_bytes` 133 ns, the round trip 658 ns, `to_string` 344 ns. The second pass's baseline was built from another path than its final tree, which skews its `to_bytes` rows (see Traps): compare within one pass's table only.

Allocations a corpus pass: `parse` 2,129, `decode` 3,732 (575 KB), `to_bytes` 153, `to_string` 309; the short card 13, 27, 1 and 2. A decoded value is 120 bytes and a property 192; a parsed line is 160.

The corpus is tests/corpus: 146 `.vcf` files, 437 KB, 5,876 physical lines, of which 3,537 are fold continuations; three folded `PHOTO`s hold 41% of the bytes, logical lines run a median of 25 bytes, and 96 of the 2,246 content lines are `ADR`. Look at both the corpus and the short card before keeping a change: the passes' wins split very differently between them.

## Where the time goes

`perf record -c 100000 -e cycles:u` on the harness after this pass:

- `parse` (corpus): `VcardLine::take_into` 24%, `physical_line` 12% (the line-feed scan), memmove 10%, `take_params` 7%, `_int_malloc` 6%, `take_body` 4%, `malloc_consolidate` 4%, the parameter lists' drop 3.7%.
- `parse` (card): `take_into` 20%, `take_params` 15.5%, memmove 12% (the card's moves out of `take_body` and into the item), `take_body` 7%, `finish_grow` 6%, the lines' drop 6%, `physical_line` 5%.
- `dec`, decode alone (corpus): `VcardParamNode::decode` 8.7%, `_int_malloc` 8%, `decode_items_into` 7%, `malloc_consolidate` 6%, `decode_value_kind_into` 6%, `VcardLine::decode_into` 5%, memchr's `find_sse2` 4.7% (the backslash searches), `VcardPropKind::from_name` 4.3%.
- `dec` (card): `VcardParamNode::decode` 16%, `malloc_consolidate` 12%, `decode_items_into` 8%, `decode_value_kind_into` 6%, malloc and `_int_malloc` 9% together, `finish_grow` 4%, the `N` and `ADR` component loop 3.9%.
- `encode`, `to_string` of decoded cards (corpus): memmove 19%, `escaped_at` 17%, `Display::fmt` 15%, `escape_into` 9%, `VcardValue::emit` 7%, `VcardValueOut::component` 4%, `VcardPropName::deref` 3%.
- `encode` (card): memmove 17%, `Display::fmt` 15%, `component` 15%, `escaped_at` 12%, `escape_into` 10%, `emit` 10%, `VcardPropName::deref` 5%.
- `tob`, `to_bytes` alone (corpus): `VcardLine::write_bytes` 40%, memmove 30% (the folded `PHOTO`s' pieces), `VcardParamNode::write_bytes` 9%, `VcardWirePart::write_bytes` 9%; on the card memmove is down to 6%.

The third pass removed most waits: failed store forwards (`ls_bad_status2.stli_other:u`, 100 passes) fell from 6.03M to 1.05M on the card's parse (about 12 a line to 2), from 9.20M to 1.49M on its decode alone (20 a property to 3), from 2.29M to 0.67M and from 3.57M to 1.10M on the corpus. What is left is work, the allocator and the per-card moves.

## Levers, in order

1. Allocation (the old lever 6). The public `Vec` fields fix one allocation a parameter list, a parameter's value list, a decoded parameter list and a decoded item list, and two large ones a card: the line list (16 lines of 160 bytes) and the decoded property list (192 bytes a property). Both large requests pass glibc's 1 KB threshold, so each card's first one consolidates every fastbin chunk the previous card freed: `malloc_consolidate` takes 6 to 12% of decode alone. Parse plus decode costs some 35% more than parse and decode alone added (2,061k against 623k and 902k), mostly in `_int_malloc` and `malloc_consolidate`. Look for fewer small chunks a card, or a line list that starts below 1 KB and still grows once, measured on both rows.
2. The decoded parameter. `VcardParamNode::decode` is 16% of the short card's decode alone: `Type(self.list())` copies the list's header back from `list`'s return slot (41% of its failed forwards). Inlining `list` with a caret pre-pass cost more than it saved; decode the parameter into a constant empty `VcardParam` grown in the property's list, as the property is.
3. The per-card moves. A parsed card (368 bytes) is built in `take_body`'s return slot, then copied into the iterator's item; building it straight in the item was flat (see Tried and dropped), the copy waiting on the `END` line's and the list's stores either way. criterion's round trip (`parse(..).unwrap().to_bytes()`) costs some 80 ns more than its two parts: the unwrap moves the card right after it is built.
4. The text writer. `memmove` is 16% of `to_string`: every `push_str` of a name, a parameter or a value run is a `memcpy` call. A `Vec<u8>` written with `tree::leaf::put` (as `to_bytes` is) and turned into text once by `text` (the ASCII proof, no new `unsafe`) may pay; weigh the one UTF-8 check of the whole text against the calls.
5. A structural scanner on simd-cpu, gated. The line end, the value colon and the name's semicolon are now found by inlined 16-byte block scans over the rest of the input (`tree::codec::escape::position`), which beat a `memchr` call a line. A wider kernel (AVX2, AVX-512 from `SimdCpu::current()`) only if a kernel bench on the corpus's line lengths beats the 16-byte scan; the parameter split (`VcardParamNode::take_into`) still walks a head byte by byte. If the parser must ask for positions, it asks through an I/O-free coroutine, never a trait (the user's rule for pluggable behaviour).
6. Smaller items: `VcardPropName::deref` (3% of `to_string`), the `eq_ignore_ascii_case` loops of the kind lookups (2 to 4% of decode), and the 137 shrinking `realloc`s a corpus pass of `into_boxed_str` on the unescape slow path.

## The bench rows owed

Grow benches/parse.rs, so each lever shows on short and long lines in criterion too (owed since the first pass):

- The corpus through `parse_many`, for vcard-rs and for every rival that accepts each file. Whether the rivals parse all 146 files is unchecked: run each over the corpus first and state in the row how many cards it took.
- A 3.0 card with a folded 40 KB `PHOTO` (copy one from the corpus), and a 2.1 card in quoted-printable (`CHARSET`, soft breaks).
- The decoders against the crates they replaced: `decode_base64` against base64 0.23's `STANDARD` (on input it accepts, no whitespace) and `quoted_printable()` against quoted_printable 0.5 `Robust` (on ASCII input), both added as dev-dependencies only, with the token installed by `SimdCpu::install_detected()` at the start of the group (criterion owns `main`).

## Tried and dropped

From the first pass:

- `Vec::resize_with` for the placeholder line the card parser fills: `parse` +2% against `push` (since retried and kept in the third pass, with the line's fields no longer dropped).
- `from_name` upper-casing in a byte loop rather than `copy_from_slice` then `make_ascii_uppercase`: corpus `decode` +1.4%, card `decode` +4.8%, though with 1% fewer instructions.
- `#[inline(always)]` on `decode_value` and `decode_value_kind`: flat.
- Reserving the folded value after copying the first physical line (a reallocation): -1.9% with 7% more instructions; sizing before the first copy kept instead (-3.7%).

From the second:

- `str::from_utf8` behind an `#[inline(never)]` helper returning `Option<&str>` in registers: decode alone +1.5%, the wait moved to the next copy of the returned `Cow`.
- `#[inline(always)]` on `VcardLine::decode`, the 504-byte property then built in its slot: +0.8% and +1.6%.
- `#[inline(always)]` on `build`: card parse +6.3% with 2.8% fewer instructions.
- The parameters pushed first and filled where they land: +0.8% (kept in the third pass, once nothing else in the line was staged).
- The wire shape recorded straight into the slot, not copied into `build`: parse -0.7%, card +0.9% (the third pass's common line records no wire shape at all).
- An inlined 16-byte scan of a physical line's first 64 bytes before memchr: parse +0.8%, card -1.1%, 3.1% more instructions (the third pass's scans over the rest of the input, finding the colon and the semicolon too, paid).
- With the box: `#[inline(always)]` on `decode_value` and `decode_value_kind` (decode alone +6.8%); the value decoded before the parameters (+7.4%).

From the third (each against the build before it, layout 0, 7 interleaved runs):

- The `BEGIN` line passed to `take_body` by value, with `parse_bare` also growing its list by `resize_with`: parse +3.0%, card +3.9%, 543 more instructions a card (the line copied right after its stores, and `resize_with` no longer inlined with two callers). Passed by reference, `parse_bare` back on `push`: flat on `parse_many` (+1.8%, -0.6%), kept because `VcardCst::parse` no longer takes its first line twice.
- `#[inline]` on `Components::next`: LLVM kept the call, flat. `#[inline(always)]` kept (card decode alone -5.2%).
- `list` inlined, borrowing every value after a pre-pass for carets: card decode alone +2.7%, 1% more instructions.
- The card built in the iterator's item, `take_body` writing the rest through a reference and the trailing blank lines read first: flat to +2.5% (card parse).
- A first in-place decode with the slow paths boxed, before the kind lookups and the splits were fixed: decode alone went from -5.5% to -0.3% as the waits moved into `from_name`'s `memcmp` (13% of cycles) and `str::split`'s `bcmp`; the lookups and the byte splits then took it to -7%.
- The decoded parameter list extended without `reserve_exact`: corpus decode rows 1 to 2.5% faster, the card's parse and decode 1.5 to 3% slower, and four slots allocated for a list of one; the exact list kept.

## Traps

- One binary measured twice in single runs can move 5%: only medians of interleaved runs decide. Two builds of the same code at different layouts differ by up to 2.6% a row; an unaligned against an aligned build moved `decode` 10%.
- Layout. Panic messages embed source paths, so the same code built from the scratchpad copy and from the repository gets a different .rodata, which shifts .text: the `to_bytes` rows read 20 to 60% slower in one such build (front-end bound: `de_no_dispatch_per_slot.no_ops_from_frontend` 468M against 122M, op-cache misses 5.8M against 3.2M). A `-C metadata` seed changes symbol names, not the layout. The harness's `build.sh` remaps the baseline copy's path to the repository's (`--remap-path-prefix`), which gives byte-identical section and function addresses for the same code, and builds four layouts by remapping the path to prefixes of four lengths; a result is kept when it holds in all four.
- A stale build: copying older files over the baseline copy keeps their old mtimes, and cargo then reuses the artifacts of what was there before. Delete the target directory when swapping the copy's contents, and check the instruction counts differ where they should.
- Another process on CPU 20, core 4's SMT sibling, makes a row bimodal; another session pinned to CPU 4 itself skews every row it overlaps (2026-11-03: the ical-rs pass measured on CPU 4 from 08:47 to 09:46 and moved to core 12). Check `ps -eo psr,pcpu,args` for CPUs 4 and 20 and `uptime` before a measuring run, coordinate cores with other sessions over SendMessage, and never measure while fuzzers or the differential run go.
- Criterion under load misleads, even interleaved: read criterion for the rival ratios; decide with the cycle harness.
- A failed-forward site removed often moves the wait to the next load (five times in the second pass, again in the third when the decode's copies went and `from_name`'s surfaced). Remove every wait on a path in one change, then measure. Keep a change only for its cycles, not for a lower stall count or fewer instructions.
- LLVM stages a `Cow` (any multi-word enum) that two arms produce in a stack temporary and moves it with a 16-byte load, even for an arm that builds it from registers; and a function returning more than two words returns through memory. Store each arm's value into its destination separately (two `push`es, or a fast arm stored inline and a slow arm out of line that writes the destination itself), or return a `Box<str>` (two registers).
- `str::split(char)` and `split_once(char)`: unless inlined with a constant char, the char searcher confirms every `memchr` hit with a `bcmp` call (`PlainPieces` in tree/value/node.rs replaces them on the decode path).
- `Vec::extend` into an empty list reserves at least four elements; `collect` of an exact-size iterator allocates exactly.
- `#[inline]` hints were ignored where `#[inline(always)]` moved cycles 5%, sometimes the wrong way: measure each.
- A block-scan predicate must join `==` tests with `|`: `matches!` compiles to a range check and a branch per byte, and LLVM does not vectorize it. A `u32` mask over 32-byte blocks compiled to four-byte loads; 16-byte blocks with a `u16` mask compile to `pcmpeqb`, `por`, `pmovmskb`.
- perf is user space only here (`perf_event_paranoid` 2): `cycles:u`, `instructions:u`, `ls_bad_status2.stli_other:u`, `ex_no_retire.load_not_complete:u`, `ex_ret_brn_misp:u` work, call graphs fail, but branch stacks work (`perf record -b`, then `perf report -b --sort symbol_from,symbol_to`) and name a function's callers, memmove's and memcmp's among them. Sample at `-c 200000`: at `-c 10000` the kernel throttles and the counts collapse. callgrind (`nix-shell -p valgrind`, then `callgrind_annotate --tree=caller`) names the callers of `memcpy` and `malloc` exactly (but has no AVX-512).
- The cursor reads in the differential run only see unsplit values unless a line is edited first; the run below edits component 2 of every line for that.
- A script that copies the binary after `cargo build` must stop on a failed build, or it measures a stale binary; piping it into `tail` hides its exit status, so check its `built` line first.
- Edit code with the Edit tool, not `perl` or `sed` substitutions (a `|` delimiter turns `\|` into an alternation; braces in a replacement end a `s{}{}`). Never recover a file with git: stop and report. Copy a file aside before an experiment that may be reverted.
- Never edit src/ while a background build, Miri or fuzz run may still compile it. Run fuzzers, Miri and the differential run away from the measuring core.
- The merge fuzz target crashes on fuzz/artifacts/merge/crash-8b19b70e3f26662eb1f5db06655ac84eb0cf0614 (a merged card that does not reparse), on the baseline too: a merge bug for its own change, not a speed regression. Run it with `-fork=3 -ignore_crashes=1` and check any new artifact against the baseline.
- `cargo doc` with `RUSTDOCFLAGS='--cfg docsrs'` fails on stable (E0554); check docs with `RUSTDOCFLAGS=-Dwarnings` alone. A public module's `//!` header may not link a `pub(crate)` item.

## How to measure

Keep the harness outside the repo, in a scratchpad: one crate for the working tree (h-new) and one for a copy of the tree before the pass (h-old, its path dependency on a `base/` copy of src, benches, examples, Cargo.toml and Cargo.lock). Cargo.toml:

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

src/main.rs, run as `vcyc <work> <passes>`, `<work>` one of `one` (`VcardCst::parse` of each file), `parse` (`parse_many`), `decode` (parse then `decode`), `bytes` (parse then `to_bytes`), `dec` (`decode` of cards parsed beforehand), `tob` (`to_bytes` of cards parsed beforehand), `encode` (`to_string` of cards decoded beforehand), or any of them prefixed `card-` for the criterion card parsed 300 times a pass:

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
    if work == "one" {
        for _ in 0..n {
            for f in corpus {
                let _ = black_box(VcardCst::parse(black_box(f.as_slice())));
            }
        }
        return;
    }
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

Build each version aligned, in its own target directory per layout, and copy the binary aside under its own name, stopping on a failed build (`build.sh <new|old> <name> [layout]`, from the scratchpad; `P` is the scratchpad):

```sh
set -e
R=/home/soywod/code/pimalaya/vcard
cd $P/h-$1
case ${3:-0} in
  0) TO=$R ;;
  1) TO=/v ;;
  2) TO=/$(printf 'v%.0s' $(seq 79)) ;;
  3) TO=/$(printf 'v%.0s' $(seq 179)) ;;
esac
if [ $1 = old ]; then FROM=$P/base; else FROM=$R; fi
T=$P/h-$1/target-aligned-${3:-0}
RUSTFLAGS="-C llvm-args=-align-all-functions=6 -C llvm-args=-align-loops=64 --remap-path-prefix=$FROM=$TO --remap-path-prefix=$P/h-$1=$P/h-new" \
  nix-shell $R/shell.nix --run "CARGO_TARGET_DIR=$T cargo build --release -q"
cp $T/release/vcyc $P/bin/$2 && echo built
```

Count cycles at two pass counts and take the difference, which removes the corpus loading; compare versions by interleaving 7 runs of each binary (11 for a close call), reading medians and minimums with the instruction count beside (`multi.sh <runs> "<binaries>" <work...>`, from the scratchpad; the first binary is the reference):

```sh
cd $P/bin
r=$1; bins=($2); shift 2
one() {
  for n in 20 120; do
    taskset -c 4 perf stat -x, -e cycles:u,instructions:u ./$1 $2 $n 2>&1 >/dev/null | cut -d, -f1 | tr '\n' ' '
    echo
  done | awk 'NR==1{c=$1;i=$2} NR==2{printf "%.1f %.1f\n", ($1-c)/100e3, ($2-i)/100e3}'
}
for w in "$@"; do
  for b in "${bins[@]}"; do : > m.$b.$w.txt; done
  for i in $(seq $r); do for b in "${bins[@]}"; do one $b $w >> m.$b.$w.txt; done; done
  line=$(printf "%-12s" $w); first=""
  for b in "${bins[@]}"; do
    med=$(sort -n m.$b.$w.txt | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
    mn=$(sort -n m.$b.$w.txt | head -1 | cut -d' ' -f1)
    ins=$(head -1 m.$b.$w.txt | cut -d' ' -f2)
    [ -z "$first" ] && first=$med
    line="$line $(awk -v m=$med -v n=$mn -v f=$first -v i=$ins 'BEGIN{printf "| %7.1f %+5.1f%% min %7.1f i %7.1f", m, (m-f)/f*100, n, i}')"
  done
  echo "$line"
done
```

Screen a change on layout 0 against the build before it (`multi.sh 7 "prev-L0 new-L0" ...`); for the pass's table, build the baseline and the final tree in all four layouts and interleave the eight (`multi.sh 7 "old-L0 new-L0 old-L1 new-L1 old-L2 new-L2 old-L3 new-L3" ...`).

To count allocations, a second harness wraps `System` in a counting `#[global_allocator]` (allocations, reallocations, bytes and a power-of-two size histogram, counted only inside a closure) and reports `parse`, `decode`, `to_bytes` and `to_string` for one corpus pass and one card.

To name stalling copies, sample the failed forwards on the debug-info build and annotate the symbol as `perf report` prints it, and name a copy's callers from branch stacks:

```sh
nix-shell -p perf --run "taskset -c 4 perf record -c 1000 -e ls_bad_status2.stli_other:u ./vcyc-new card-dec 300 \
  && perf annotate --stdio -s '<vcard::tree::param::node::VcardParamNode>::decode'"
nix-shell -p perf --run "taskset -c 4 perf record -c 20000 -e cycles:u -b ./vcyc-new card-dec 200 \
  && perf report -b --sort symbol_from,symbol_to --stdio | grep memcmp"
```

For criterion, build the baseline copy's bench (its path remapped to the repository's) and the working tree's in their own target directories (`cargo bench --bench parse --no-run`), run the two executables alternately with `--bench --output-format bencher` on core 4, three times each, and take the medians.

## The differential run

For any parser or model change: copy the previous src/ and Cargo.toml into a scratch crate renamed `vcard-old` (lib name `vcard_old`, its `[features]` and `[dependencies]` kept, the `[[bench]]`, `[[example]]` and `[dev-dependencies]` sections dropped), then for the corpus and 300 to 600 rounds of random edits of it (a byte removed, replaced or inserted from an alphabet of CR, LF, space, tab, `:`, `;`, `,`, `"`, `=`, `\`, `^`, `ENCODING=QUOTED-PRINTABLE`, `CHARSET=`, `BEGIN:VCARD`, `END:VCARD`, `VERSION:2.1`, `VERSION:3.0`, `VERSION:4.0`, `item1.`, non-ASCII bytes, `\r\n `, `\r\n\t`, `=\r\n`, the escapes `\n`, `\N`, `\,`, `\;`, `\\`, the carets `^n`, `^'`, `^^`, `;VALUE=uri`, `;TYPE="a,b"`, whole `ADR`, `N`, `GEO`, `GENDER` and `X-` lines, and chunks copied from elsewhere in the file; files over 4 KB cut to a random 4 KB window), through `parse` and `parse_many`, assert:

- `format!("{:?}")` of both results match, trees and errors;
- `to_bytes` and `to_string` of both match;
- `decode` of both match (`Debug`), and for the decoded cards: `to_string`, `encode` (`Debug`, `to_bytes`, and its `to_string` against the direct `to_string`), and every property's `encode` under each of the three escapers;
- the same on the decoded model salted under each version: a salt (separators, a backslash, a line break, carets, a quote, non-ASCII, `\r\n`) put at the front, the back or both ends of every string a model can hold, a group set on every fourth property and empty lists filled, through a walker written once as a macro for both crates;
- every cursor read (`text`, `bytes`, `list`, `component` 0, 1, 2 and 7, `quoted_printable`, `charset`) on every line matches;
- then, after `set_component(2, &["x,y", "z;"])` on every line of both, `to_bytes`, `to_string`, `decode`, the model checks and every cursor read match again.

Save a failing input before panicking. Plant a break in the old copy, in the code the change touches, and see the run fail before trusting it. The run takes under two minutes for 600 rounds.

## Before landing a pass

`cargo test` under default, all and no features; clippy `--all-targets -- -D warnings` under all and no features; `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features`; `cargo build --no-default-features` depending on nothing and warning-free; the unit tests and the corpus, codecs and encoding tests under Miri when `unsafe` changed (`nix-shell ~/code/pimalaya/quoted-printable-codec/fuzz/shell.nix --run "cargo miri test --lib"` from this repo, its nightly carrying Miri); the fuzz targets for ten minutes each (`nix-shell fuzz/shell.nix --run "cd fuzz && cargo fuzz run -O parse -- -max_total_time=600"` on one core, merge forked as above on three others), seeded from a copy of tests/corpus in fuzz/corpus/parse and fuzz/corpus/merge; `cargo fmt`; the differential run; the cycle harness on every workload against the pass's baseline, in four layouts; a criterion run interleaved with the baseline's; benches/README.md numbers; CHANGELOG; a log entry; this proposal rewritten; then ask the user before committing.
