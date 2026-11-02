---
cairn: change
id: the-lines-built-in-place
status: landed
created: 2026-11-02
---

# The lines built in place

The handoff for the first speed pass over the parser and the decoded model, written on 2026-11-02 so a fresh session can take the lead on its own. Rewrite it at the end of each pass, as mime-rs does with its cairn/changes/the-scratch-area/proposal.md: where things stand, where the time goes, the levers left, what was tried and dropped, the traps.

## Read first

- This repo's AGENTS.md and cairn/spec/parsing.md: round-trip fidelity, the serialization fixpoint and line normalisation are the requirements a speed change must not move.
- ~/code/pimalaya/mime/cairn: the three passes this one copies (log/2026-11-02-the-tree-built-in-place.md, the-steps-inlined, the-state-carries-no-data) and changes/the-scratch-area/proposal.md, whose measuring method, traps and tried-and-dropped list apply here almost word for word.
- cairn/log/2026-11-02-the-simd-codecs.md: base64 and quoted-printable moved to base64-codec and quoted-printable-codec the same day. That change is in the working tree, uncommitted: the baseline of this pass is the working tree, not HEAD.
- cairn/changes/an-edited-line-is-folded: a draft that will rewrite src/tree/wire.rs. Leave wire.rs's behaviour alone; a speed change there must keep its output byte for byte.

## Goal and rules

Make parsing, decoding and serializing faster with no behaviour change, in mime-rs's way:

- A change is kept only for its cycles, measured as below; a change that is flat goes, even if it looks cleaner to a profiler. Smaller source is a goal too: the user's bar for the codecs was beating every rival row with the least code.
- Every parser change gets the differential run below and a planted break that the run catches.
- No new dependency for the core. `cargo build --no-default-features` must stay dependency-free; the `parser` feature may trade memchr for simd-cpu (lever 6), nothing else.
- Each pass ends with: README.md and benches/README.md numbers, a log entry, this proposal rewritten as the next handoff, and a commit `perf: <what>` only once the user says yes to "commit now?" (no git write before that, no AI trailer in the message).

## Where things stand

Criterion on 2026-11-02, `taskset -c 4 cargo bench --bench parse`, load 0.7, Zen 5 (AMD Ryzen AI Max+ 395), rustc 1.96.1 from the devshell. The bench parses one 16-line vCard 4.0 card:

| level | library | time | against vcard-rs |
| --- | --- | ---: | ---: |
| content lines | vparser 1 | 0.36 µs | 3.7x faster |
| content lines | **vcard-rs** `VcardCst::parse` | **1.32 µs** | |
| content lines | ical_vcard 0.5 | 2.51 µs | 1.9x slower |
| model | calcard 0.3 | 3.24 µs | 4% faster |
| model | **vcard-rs** parse + `decode` | **3.38 µs** | |
| model | vcard_parser 0.2 | 62.0 µs | 18x slower |
| own | `decode` alone | 1.91 µs | |
| own | `to_string` of the decoded card | 3.78 µs | |
| own | `to_bytes` | 0.34 µs | |
| own | parse + `to_bytes` | 1.76 µs | |

benches/README.md still says vcard-rs leads calcard by 2%; it is now 4% behind. vparser is a zero-allocation pull tokenizer that builds no tree, so it bounds the content-line row from below rather than being a like-for-like rival.

Cycles per corpus pass from the harness below (aligned build, user space, medians of 5): `parse` 1,551k cycles and 5,290k instructions, `decode` (parse then `decode`) 4,889k and 13,815k, `bytes` (parse then `to_bytes`) 2,205k and 6,964k. An unaligned build gave 1,479k, 4,394k and 2,206k: layout alone moves `decode` 10%, so always compare builds made the same way.

The corpus is tests/corpus: 145 `.vcf` files, 437 KB, 5,876 physical lines, of which 3,537 are fold continuations. Three `PHOTO`-carrying files (85 KB, 47 KB, 47 KB) hold 41% of the bytes. Logical lines run a median of 25 bytes (90th percentile 67). So the corpus pass weighs long folded base64 lines, the criterion card short unfolded ones: look at both before keeping a change.

## Where the time goes

`perf record -e cycles:u` on the harness:

- `parse`: `VcardLine::parse` 21%, `VcardCst::take_card` 10%, `memmove` 10%, `VcardLine::take` 7%, the allocator (`_int_malloc`, `malloc_consolidate`, `free`, `finish_grow`) about 15%, `VcardParamNode::parse` 5.5%, memchr 5%, `from_utf8` 3%, `into_static` 2.6%, the drop of a line 2%, `head_is_quoted_printable` 1.8%, `push_content` 1.6%, `VcardWire::prepend` 1.6%, the property-kind index vector growing 1.3%.
- `bytes`: `memmove` 16%, `VcardLine::parse` 14%, the allocator about 17%, `take` 6.7%, `take_card` 6%, `from_utf8` 4.3%, `write_bytes` 3.3%, `head_is_quoted_printable` 3.2%.
- `decode`: `memmove` 13%, `String::from_utf8_lossy` (`Utf8Chunks::next`) 10%, `VcardLine::parse` 7%, the allocator about 13%, memchr2 4%, `VcardLine::decode` 4%, `decode_component_list` 2.7%, `VcardPropKind::from_str` 2.6%, `unescape_bytes` 2%.

Failed store forwards: `perf stat -e ls_bad_status2.stli_other:u` counts about 93k a `parse` pass, against 7k for a mime-rs `structure` pass after its third pass: about 40 for each of the 2,339 logical lines. Mispredicted branches about 10.7k a pass. In the annotation, `take_card` puts 64% of its samples on one stack store and `VcardLine::parse` more than half of its own on three stack copies: `VcardLine` is 160 bytes, returned by value inside `Result<(Self, &[u8]), _>` from `take` and from `parse`, then pushed. mime-rs found the same shape: a value returned through memory and a `Vec::push` of a multi-word value staged on the stack each stall the next load.

## Levers, in order

1. Lines written in place. `take_card` grows `props` by one slot and the tokeniser fills it (mime-rs measured `Vec::resize_with` writing in place where `push` stages and copies); `take` and `parse` stop returning a 160-byte line by value. Check also `VcardWire`: every fold calls `wire.fold`, a push of a multi-word `(offset, part)` pair, `into_static` collects the parts into a new vector, and `prepend` appends the line's own parts to the tokeniser's and sorts the whole list (a stable sort, which the offset order requirement relies on); on a 3,537-fold corpus that path is a candidate for the per-line forward stalls. Sample `ls_bad_status2.stli_other:u` to name the sites before and after.
2. A folded line built once. `take` copies every continuation into a `Vec`, parses it borrowed, then `into_static` copies the name, each parameter and each value leaf again. Hand the owned buffer to the value (it is most of the bytes, the base64 of a `PHOTO`) and copy only the small head, or take the leaves from the buffer by offset.
3. One pass over the head. Each line runs `value_colon` (a byte loop), `head_is_quoted_printable` (`value_colon` again, then a split on `;`), `from_utf8` on the head, then `param_semicolon` per parameter (a byte loop again), and `take` runs `head_is_quoted_printable` once more on a line ending in `=`. Find the colon, the semicolons outside quotes and the quoted-printable declaration in one walk (`memchr3` on `:`, `;` and `"`), validate UTF-8 once.
4. `decode`. Validate with core's `from_utf8` first and go lossy only on failure (mime-rs did this for its text leaves); `VcardPropKind::from_str` is a chain of about 50 `eq_ignore_ascii_case` arms tried in order: compare the canonical upper-case spelling exactly first, or dispatch on length and first byte (mime-rs's "names compared exactly first" was one of its third pass's wins).
5. Allocations. Count them first (a counting `#[global_allocator]` around `System`, allocations, reallocations and a size histogram over one corpus pass), then cut the ones on every line: the parameter vector of a parameterless line, the owned line-ending leaf of a folded line (`eol_leaf` allocates for `\r\n`, which could be a `&'static str`), the value's component vectors.
6. A structural scanner on simd-cpu, last. memchr here is built without its `std` feature, so it never dispatches to AVX2 at run time: every search runs `find_sse2`, and each call pays its setup on 25-byte lines. One SIMD pass over a card classifying `\n`, `:`, `;`, `"`, `\\` and `,` into bit masks (AVX-512, AVX2, NEON and portable backends from `SimdCpu::current()`), read by the line splitter and the head and value splits, could replace memchr in the `parser` feature. Gate: a kernel bench on the corpus's line lengths must beat memchr's per-call scans first, as mime-rs gated its own SIMD answerers. If the parser must ask for positions, it asks through an I/O-free coroutine, never a trait (the user's rule for pluggable behaviour). The codecs' traps apply: see the memory notes on base64-codec and quoted-printable-codec, and their cairns (the-gathered-blocks, the-column-chain).

## The bench

Grow benches/parse.rs before the first lever, so each one shows on short and long lines:

- The corpus through `parse_many`, for vcard-rs and for every rival that accepts each file. Whether the rivals parse all 145 files is unchecked: run each over the corpus first and state in the row how many cards it took.
- A 3.0 card with a folded 40 KB `PHOTO` (copy one from the corpus), and a 2.1 card in quoted-printable (`CHARSET`, soft breaks).
- The decoders against the crates they replaced: `decode_base64` against base64 0.23's `STANDARD` (on input it accepts, no whitespace) and `quoted_printable()` against quoted_printable 0.5 `Robust` (on ASCII input), both added as dev-dependencies only, with the token installed by `SimdCpu::install_detected()` at the start of the group (criterion owns `main`).

## Tried and dropped

Nothing yet. List every lever measured and not kept here with its numbers, as mime-rs does: it is what stops the next session from trying it again.

## Traps

- One binary measured twice in single runs can move 5%: only medians of interleaved runs decide. Two builds of near-identical code still differ by up to 1.5% from layout; an unaligned against an aligned build moved `decode` 10% here.
- Criterion against its own saved run misleads once the machine's load changed: build the previous tree's bench in its own target directory and interleave the two.
- A failed-forward site removed often moves the wait to the next load. Keep a change only for its cycles, not for a lower stall count.
- `#[inline]` hints were ignored in mime-rs where `#[inline(always)]` moved cycles 4%, sometimes the wrong way: measure each.
- perf is user space only here (`perf_event_paranoid` 2): `cycles:u`, `instructions:u`, `ls_bad_status2.stli_other:u`, `ex_ret_brn_misp:u` work, call graphs fail. callgrind (`nix-shell -p valgrind`) gives exact counts and callers (but has no AVX-512).
- A script that copies the binary after `cargo build` must stop on a failed build, or it measures a stale binary.
- Edit code with the Edit tool, not `perl` or `sed` substitutions (a `|` delimiter turns `\|` into an alternation and once rewrote the wrong line in mime-rs). Never recover a file with git: stop and report.
- Never edit src/ while a background build or fuzz run may still compile it. Run fuzzers and Miri away from the measuring core (`taskset -c 2-11`).
- `cargo doc` with `RUSTDOCFLAGS='--cfg docsrs'` fails on stable (E0554); check docs with `RUSTDOCFLAGS=-Dwarnings` alone. The devshell has no thumbv6m std; a throwaway fenix shell combining `fenix.targets.thumbv6m-none-eabi.stable.rust-std` builds it.

## How to measure

Keep the harness outside the repo, in a scratchpad, its path dependency pointing at the working tree. Cargo.toml:

```toml
[package]
name = "vcyc"
version = "0.0.0"
edition = "2024"

[dependencies]
vcard = { package = "vcard-rs", path = "/home/soywod/code/pimalaya/vcard" }

[profile.release]
codegen-units = 1
debug = 1
```

src/main.rs, run as `vcyc <parse|decode|bytes> <passes>`:

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
    let mut a = std::env::args().skip(1);
    let work = a.next().unwrap();
    let n: usize = a.next().unwrap().parse().unwrap();
    let mut corpus = Vec::new();
    walk(Path::new("/home/soywod/code/pimalaya/vcard/tests/corpus"), &mut corpus);
    for _ in 0..n {
        for f in &corpus {
            for card in VcardCst::parse_many(black_box(f.as_slice())) {
                let Ok(card) = card else { continue };
                match work.as_str() {
                    "parse" => { black_box(&card); }
                    "decode" => { black_box(card.decode()); }
                    "bytes" => { black_box(card.to_bytes()); }
                    _ => panic!(),
                }
            }
        }
    }
}
```

Add a `SimdCpu::install_detected()` (simd-cpu at the rev vcard-rs pins, `features = ["std"]`) once a workload decodes base64 or quoted-printable.

Build it aligned, each version in its own target directory, and copy the binary aside under its own name, stopping on a failed build:

```sh
RUSTFLAGS='-C llvm-args=-align-all-functions=6 -C llvm-args=-align-loops=64' \
  nix-shell /home/soywod/code/pimalaya/vcard/shell.nix \
  --run "CARGO_TARGET_DIR=$PWD/target-aligned cargo build --release" \
  && cp target-aligned/release/vcyc vcyc-new
```

Count cycles at two pass counts and take the difference, which removes the corpus loading; compare two versions by interleaving 5 to 7 runs of each and comparing medians, the instruction count beside (`ab.sh vcyc-old vcyc-new 7 parse decode bytes`):

```sh
A=$1; B=$2; r=$3; shift 3
one() {
  for n in 100 600; do
    nix-shell -p perf --run "taskset -c 4 perf stat -x, -e cycles:u,instructions:u $1 $2 $n" 2>&1 >/dev/null | cut -d, -f1 | tr '\n' ' '
    echo
  done | awk 'NR==1{c=$1;i=$2} NR==2{printf "%.1f %.1f\n", ($1-c)/500e3, ($2-i)/500e3}'
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
  && perf annotate --stdio -s '<vcard::tree::cst::VcardCst>::take_card'"
```

For a baseline, copy the tree before the pass (src, tests, benches, Cargo.toml, Cargo.lock) into the scratchpad and point a second harness at it.

## The differential run

For any parser or model change: copy the previous src/ and Cargo.toml into a scratch crate renamed `vcard-old` (lib name `vcard_old`, its `[features]` and `[dependencies]` kept, examples and benches dropped), then for the corpus and 300 rounds of random edits of it (a byte replaced, removed or inserted from an alphabet of CR, LF, space, tab, `:`, `;`, `,`, `"`, `=`, `\`, `ENCODING=QUOTED-PRINTABLE`, `CHARSET=`, `BEGIN:VCARD`, `END:VCARD`, `VERSION:2.1`, `item1.`, non-ASCII bytes, and chunks copied elsewhere), through `parse` and `parse_many`, assert:

- `format!("{:?}")` of both trees match, and so do the errors;
- `to_bytes` gives the input back where the old one did, and both outputs match;
- `decode` of both match (`Debug`), and so do `to_string` of the decoded cards;
- every cursor read (`text`, `bytes`, `list`, `component`, `quoted_printable`, `charset`) on every line matches.

Plant a break in the old copy, in the code the change touches, and see the run fail before trusting it.

## Before landing a pass

`cargo test` under default, all and no features; clippy `--all-targets -- -D warnings` under all and no features; `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features`; `cargo build --no-default-features` depending on nothing; the fuzz targets for ten minutes each (`taskset -c 2-11 nix-shell fuzz/shell.nix --run "cargo fuzz run parse -- -max_total_time=600"`, then `merge`), seeded from a copy of tests/corpus in fuzz/corpus/; `cargo fmt`; the differential run; a criterion run of the whole bench, interleaved with the previous tree's when the load is not quiet; README.md and benches/README.md numbers; a log entry; this proposal rewritten; then ask the user before committing.
