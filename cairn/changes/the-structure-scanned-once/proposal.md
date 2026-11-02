---
cairn: change
id: the-structure-scanned-once
status: active
created: 2026-11-02
---

# The structure scanned once

The handoff for the second speed pass over the parser and the decoded model, written at the end of [the-lines-built-in-place](../archive/the-lines-built-in-place/proposal.md) so a fresh session can take the lead on its own. Rewrite it at the end of each pass that takes it up, as mime-rs does with its cairn/changes/the-scratch-area/proposal.md: where things stand, where the time goes, the levers left, what was tried and dropped, the traps.

## Read first

- This repo's AGENTS.md and cairn/spec/parsing.md: round-trip fidelity, the serialization fixpoint and line normalisation are the requirements a speed change must not move.
- cairn/log/2026-11-02-the-lines-built-in-place.md: what the first pass changed, each change with its cycles, and what it dropped.
- ~/code/pimalaya/mime/cairn: its passes (log/2026-11-02-the-tree-built-in-place.md, the-steps-inlined, the-state-carries-no-data) and changes/the-scratch-area/proposal.md, whose method and traps apply here.
- cairn/changes/an-edited-line-is-folded: a draft that will rewrite src/tree/wire.rs. A speed change there must keep its output byte for byte.

## Goal and rules

Make parsing, decoding and serializing faster with no behaviour change, in mime-rs's way:

- A change is kept only for its cycles, measured as below; a change that is flat goes, even if it looks cleaner to a profiler. Smaller source is a goal too.
- Every parser or model change gets the differential run below.
- No new dependency for the core. `cargo build --no-default-features` must stay dependency-free; the `parser` feature may trade memchr for simd-cpu (lever 1), nothing else.
- No `unsafe` without the user's yes: the crate has none today.
- Each pass ends with: benches/README.md numbers, a log entry, this proposal rewritten as the next handoff, and a commit `perf: <what>` only once the user says yes to "commit now?" (no git write before that, no AI trailer in the message).

## Where things stand

The first pass took corpus `parse` from 2,427k to 1,453k cycles a pass, `decode` from 7,190k to 4,092k and `bytes` from 3,529k to 1,982k; the 16-line criterion card parsed 300 times went from 2,823k to 1,819k (`parse`), 7,912k to 5,172k (`decode`) and 3,773k to 2,219k (`bytes`). Instructions a corpus pass now: `parse` 4,082k, `decode` 8,273k, `bytes` 5,019k. Failed store forwards 19.8k a `parse` pass, mispredicted branches 2.7k. These were taken under a load of 12 to 24 from another session; on a quiet machine expect about two thirds of these cycle counts. Criterion under that load: `VcardCst::parse` 2.32 µs, vparser 1.00, ical_vcard 6.29; parse + `decode` 6.64 µs, calcard 8.56.

The corpus is tests/corpus: 146 `.vcf` files, 437 KB, 5,876 physical lines, of which 3,537 are fold continuations; three folded `PHOTO`s hold 41% of the bytes, and logical lines run a median of 25 bytes. Look at both the corpus and the short card before keeping a change: the first pass's wins split very differently between them.

## Where the time goes

`perf record -e cycles:u` on the harness after the first pass:

- `parse`: `VcardLine::take_into` 20% (physical line split, colon scan, fold join), `VcardLine::build` 15% (head UTF-8 check, parameter walk, node fills), memchr's `find_sse2` 11.5%, `memmove` 9%, `from_utf8` 5.7%, `_int_malloc` 5.5%, `take_card` 3.4%, the drops of the parameter lists and lines about 6%, `push_content` 2.6%, `finish_grow` 2.3%.
- `decode`: `from_utf8` 8.6%, `memmove` 8.6%, `take_into` 8.4%, `_int_malloc` 7.6%, `build` 6.8%, `find_sse2` 5.1%, core's `memchr_aligned` 4% (the `contains(&b'\\')` in the unescapers), `unescape_bytes` 3.4%, `VcardLine::decode` 3.3%, `malloc_consolidate` 3.1%, `memchr2` 2.6%, `VcardPropKind::from_name` 2.2%.
- `bytes`: `take_into` 16.6%, `memmove` 15.9%, `build` 10.7%, `find_sse2` 9.5%, `write_bytes` 7.4%, `from_utf8` 5.3%.

## Levers, in order

1. A structural scanner on simd-cpu, gated. memchr is built without its `std` feature here, so it never dispatches to AVX2 at run time: every search runs `find_sse2`, and each call pays its setup on 25-byte lines. The first pass made it worse for folded lines: the continuations are now scanned twice, once to size the value and once to copy it. Two steps. First, read the folds once: the sizing scan records the fold offsets in the wire shape already, so the copy loop can take each continuation's length from the next offset and skip its own `memchr` (mind the slow path, the line ending after each continuation and a final line with no break). Then, gated by a kernel bench on the corpus's line lengths that must beat memchr's per-call scans, one SIMD pass over a card classifying `\n`, `:`, `;`, `"`, `\\` and `,` into bit masks (AVX-512, AVX2, NEON and portable backends from `SimdCpu::current()`), read by the line splitter and the head and value splits. If the parser must ask for positions, it asks through an I/O-free coroutine, never a trait (the user's rule for pluggable behaviour). The codecs' traps apply: see the base64-codec and quoted-printable-codec cairns (the-gathered-blocks, the-column-chain).
2. The folded value laid out from its node. `to_bytes` writes a folded line's logical bytes, copies them again past themselves with the wire pieces, then moves them down: a 40 KB `PHOTO` is copied three times. Write the head (name, parameters, colon) and lay the wire shape out over two slices, the head and the value node's raw bytes, so the value is copied once. Pieces inside the head are rare (blank lines and indentation sit at offset 0, a fold before the colon only on the slow path) but must still come out right.
3. Allocations. Count them first (a counting `#[global_allocator]` around `System` in the harness: allocations, reallocations and a size histogram over one corpus pass and one card), then cut the ones on every line. Left after the first pass: the parameter list of a line with parameters and one value list per parameter (the public `Vec` fields fix their type), the decoded model's lists, the wire list of a line with a blank line before it.
4. The stalls left in `take_into` and `build`. Sample `ls_bad_status2.stli_other:u` on the debug-info build (19.8k a pass) and annotate both, as the first pass did for `take_card`: the placeholder line pushed for each line is still staged and copied by `memcpy` (`resize_with` measured worse), and `build` fills the slot field by field.
5. The decoded property's size. `VcardProp` is about 500 bytes because `VcardValue::Adr` holds `VcardAdr`'s 18 vectors inline; each line's decode still copies it once from `decode_value` into the property. Boxing the address is an API break: ask the user before trying it.
6. UTF-8 checks. `from_utf8` runs on every head and every decoded value; on short pieces its call costs more than its scan. Without `unsafe` there is no way around the check itself, only around calling it twice on the same bytes; with the user's yes, an ASCII proof folded into a scan the code already does (the colon scan for a head, the backslash scan for a value) could stand in for it.

## The bench rows owed

Grow benches/parse.rs, so each lever shows on short and long lines in criterion too:

- The corpus through `parse_many`, for vcard-rs and for every rival that accepts each file. Whether the rivals parse all 146 files is unchecked: run each over the corpus first and state in the row how many cards it took.
- A 3.0 card with a folded 40 KB `PHOTO` (copy one from the corpus), and a 2.1 card in quoted-printable (`CHARSET`, soft breaks).
- The decoders against the crates they replaced: `decode_base64` against base64 0.23's `STANDARD` (on input it accepts, no whitespace) and `quoted_printable()` against quoted_printable 0.5 `Robust` (on ASCII input), both added as dev-dependencies only, with the token installed by `SimdCpu::install_detected()` at the start of the group (criterion owns `main`).

## Tried and dropped

- `Vec::resize_with` for the placeholder line the card parser fills: `parse` +2% against `push`.
- `from_name` upper-casing in a byte loop rather than `copy_from_slice` then `make_ascii_uppercase`: corpus `decode` +1.4%, card `decode` +4.8%, though with 1% fewer instructions.
- `#[inline(always)]` on `decode_value` and `decode_value_kind`: flat.
- Reserving the folded value after copying the first physical line (a reallocation): -1.9% with 7% more instructions; sizing before the first copy kept instead (-3.7%).

## Traps

- One binary measured twice in single runs can move 5%: only medians of interleaved runs decide. Two builds of near-identical code still differ by up to 1.5% from layout; an unaligned against an aligned build moved `decode` 10%.
- Criterion under load misleads, even interleaved: on 2026-11-02 at a load of 24 its `to_string` and round-trip rows were bimodal (2.67 µs then 4.73 µs for one binary) and the round-trip read faster than the parse alone. Read criterion only for the rival ratios; decide with the cycle harness.
- A failed-forward site removed often moves the wait to the next load. Keep a change only for its cycles, not for a lower stall count. Fewer instructions can still be slower (the `from_name` loop).
- `#[inline]` hints were ignored in mime-rs where `#[inline(always)]` moved cycles 4%, sometimes the wrong way: measure each.
- perf is user space only here (`perf_event_paranoid` 2): `cycles:u`, `instructions:u`, `ls_bad_status2.stli_other:u`, `ex_ret_brn_misp:u` work, call graphs fail. callgrind (`nix-shell -p valgrind`, then `callgrind_annotate --tree=caller`) names the callers of `memcpy` and `malloc` exactly (but has no AVX-512).
- The cursor reads in the differential run only see unsplit values unless a line is edited first; the run below edits component 2 of every line for that.
- A script that copies the binary after `cargo build` must stop on a failed build, or it measures a stale binary.
- Edit code with the Edit tool, not `perl` or `sed` substitutions (a `|` delimiter turns `\|` into an alternation). In a perl script, a later `s///` resets `$1`: copy captures out first. Never recover a file with git: stop and report.
- Never edit src/ while a background build or fuzz run may still compile it. Run fuzzers and the differential run away from the measuring core (`taskset -c 5-11`), and never criterion while they run.
- The merge fuzz target crashes on fuzz/artifacts/merge/crash-8b19b70e3f26662eb1f5db06655ac84eb0cf0614 (a merged card that does not reparse), on the baseline too: a merge bug for its own change, not a speed regression. Expect it until that change lands.
- `cargo doc` with `RUSTDOCFLAGS='--cfg docsrs'` fails on stable (E0554); check docs with `RUSTDOCFLAGS=-Dwarnings` alone.

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

src/main.rs, run as `vcyc <work> <passes>`, `<work>` one of `parse`, `decode`, `bytes`, `encode` (`to_string` of the decoded cards), or any of them prefixed `card-` for the criterion card parsed 300 times a pass:

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
    if work == "encode" {
        let csts: Vec<_> = corpus.iter().flat_map(|f| VcardCst::parse_many(f.as_slice()).flatten()).collect();
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
  && cp target-aligned/release/vcyc bin/vcyc-new
```

Count cycles at two pass counts and take the difference, which removes the corpus loading; compare two versions by interleaving 7 runs of each and comparing medians, the instruction count beside (`nix-shell -p perf --run "./ab.sh vcyc-old vcyc-new 7 parse decode bytes card-parse card-decode card-bytes"`, from bin/):

```sh
A=$1; B=$2; r=$3; shift 3
one() {
  for n in 20 120; do
    taskset -c 4 perf stat -x, -e cycles:u,instructions:u ./$1 $2 $n 2>&1 >/dev/null | cut -d, -f1 | tr '\n' ' '
    echo
  done | awk 'NR==1{c=$1;i=$2} NR==2{printf "%.1f %.1f\n", ($1-c)/100e3, ($2-i)/100e3}'
}
for w in "$@"; do
  : > a.txt; : > b.txt
  for i in $(seq $r); do one $A $w >> a.txt; one $B $w >> b.txt; done
  for f in a b; do sort -n $f.txt | awk -v w=$w -v f=$f '{a[NR]=$1; i=$2} END{printf "%-7s %s med %7.1f kcycles %8.1f kinstr\n", w, f, a[int((NR+1)/2)], i}'; done
done
```

To name stalling copies, sample the failed forwards on the debug-info build and annotate the symbol as `perf report` prints it:

```sh
nix-shell -p perf --run "taskset -c 4 perf record -c 2000 -e ls_bad_status2.stli_other:u ./vcyc-new parse 300 \
  && perf annotate --stdio -s '<vcard::tree::line::VcardLine>::take_into'"
```

For a baseline, copy the tree before the pass (src, benches, examples, Cargo.toml, Cargo.lock) into the scratchpad and point a second harness at it. For criterion, build the baseline copy's bench and the working tree's in their own target directories (`cargo bench --bench parse --no-run`), run the two executables alternately with `--bench --output-format bencher` on core 4, three times each, and take the medians.

## The differential run

For any parser or model change: copy the previous src/ and Cargo.toml into a scratch crate renamed `vcard-old` (lib name `vcard_old`, its `[features]` and `[dependencies]` kept, the `[[bench]]`, `[[example]]` and `[dev-dependencies]` sections dropped), then for the corpus and 300 to 600 rounds of random edits of it (a byte removed, replaced or inserted from an alphabet of CR, LF, space, tab, `:`, `;`, `,`, `"`, `=`, `\`, `ENCODING=QUOTED-PRINTABLE`, `CHARSET=`, `BEGIN:VCARD`, `END:VCARD`, `VERSION:2.1`, `VERSION:3.0`, `item1.`, non-ASCII bytes, `\r\n ` and `=\r\n`, and chunks copied from elsewhere in the file; files over 4 KB cut to a random 4 KB window), through `parse` and `parse_many`, assert:

- `format!("{:?}")` of both results match, trees and errors;
- `to_bytes` and `to_string` of both match;
- `decode` of both match (`Debug`), and so do `to_string` of the decoded cards;
- every cursor read (`text`, `bytes`, `list`, `component` 0, 1, 2 and 7, `quoted_printable`, `charset`) on every line matches;
- then, after `set_component(2, &["x,y", "z;"])` on every line of both, `to_bytes`, `decode` and every cursor read match again.

Save a failing input before panicking. Plant a break in the old copy, in the code the change touches, and see the run fail before trusting it.

## Before landing a pass

`cargo test` under default, all and no features; clippy `--all-targets -- -D warnings` under all and no features; `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features`; `cargo build --no-default-features` depending on nothing; the fuzz targets for ten minutes each (`nix-shell fuzz/shell.nix --run "cd fuzz && cargo fuzz run -O parse -- -max_total_time=600"` on `taskset -c 5-7`, merge on `8-11`), seeded from a copy of tests/corpus in fuzz/corpus/parse and fuzz/corpus/merge; `cargo fmt`; the differential run; the cycle harness on all six workloads against the pass's baseline; a criterion run interleaved with the baseline's; benches/README.md numbers; a log entry; this proposal rewritten; then ask the user before committing.
