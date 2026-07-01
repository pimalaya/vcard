//! A tiny, dependency-free smoke fuzzer that runs on stable Rust, for a quick
//! look at parser robustness without the cargo-fuzz toolchain (nightly +
//! libFuzzer). It mutates a handful of seed cards and throws random bytes at the
//! parser, checking two oracles: parsing must never panic, and whatever parses
//! must serialize to a byte-stable fixpoint (its own output reparses identically).
//!
//! This is a smoke test, not a replacement for coverage-guided fuzzing: see
//! `fuzz/` for the real cargo-fuzz target. Run with an optional iteration count:
//!
//! ```sh
//! cargo run --release --example fuzz_smoke -- 1000000
//! ```

use std::panic::{AssertUnwindSafe, catch_unwind};

use vcard::tree::cst::VcardCst;

/// Real cards spanning versions and features, used as mutation seeds.
const SEEDS: &[&[u8]] = &[
    b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nN:Doe;John;;;\r\nEND:VCARD\r\n",
    b"BEGIN:VCARD\r\nVERSION:2.1\r\nFN;CHARSET=ISO-8859-1:X\r\nNOTE;ENCODING=QUOTED-PRINTABLE:caf=C3=A9\r\nEND:VCARD\r\n",
    b"BEGIN:VCARD\r\nVERSION:3.0\r\nPHOTO;ENCODING=b:Zm9vYmFy\r\nNOTE:a long\r\n note\r\nEND:VCARD\r\n",
    b"BEGIN:VCARD\r\nVERSION:2.1\r\nAGENT:\r\nBEGIN:VCARD\r\nVERSION:2.1\r\nFN:Agent\r\nEND:VCARD\r\nEND:VCARD\r\n",
    b"BEGIN:VCARD\r\nVERSION:4.0\r\nADR:;;42 Main;Town;;;US\r\nGEO:geo:37.0,-122.0\r\nEND:VCARD\r\n",
    b"cn:bare record\r\nemail:x@y.z\r\n",
];

fn main() {
    // Silence the default panic printer so a clean run stays quiet; a real crash
    // is reported by the harness itself.
    std::panic::set_hook(Box::new(|_| {}));

    let iterations: u64 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(300_000);

    let mut rng = Rng(0x9E3779B97F4A7C15);
    let mut parsed = 0u64;
    let mut non_idempotent = 0u64;
    let mut first_finding: Option<Vec<u8>> = None;

    for i in 0..iterations {
        let input = if rng.below(3) == 0 {
            let len = rng.below(256);
            (0..len).map(|_| rng.byte()).collect::<Vec<u8>>()
        } else {
            mutate(SEEDS[rng.below(SEEDS.len())], &mut rng)
        };

        match catch_unwind(AssertUnwindSafe(|| exercise(&input))) {
            Ok(outcome) => {
                if outcome.parsed {
                    parsed += 1;
                }
                if !outcome.idempotent {
                    non_idempotent += 1;
                    first_finding.get_or_insert_with(|| input.clone());
                }
            }
            Err(_) => {
                eprintln!("PANIC at iteration {i} on {}-byte input:", input.len());
                eprintln!("  {}", escape(&input));
                std::process::exit(1);
            }
        }
    }

    println!("smoke fuzz: {iterations} iterations, no panics");
    println!("  {parsed} inputs parsed as a vCard");
    println!("  {non_idempotent} non-idempotent serializations");
    if let Some(finding) = first_finding {
        println!("  first non-idempotent input: {}", escape(&finding));

        // Re-derive the two serializations from the real bytes, to show exactly
        // where the round-trip drifts.
        if let Ok(cst) = VcardCst::parse(&finding) {
            let once = cst.to_bytes();
            println!("    serialized once : {}", escape(&once));
            match VcardCst::parse(&once) {
                Ok(reparsed) => {
                    println!("    serialized twice: {}", escape(&reparsed.to_bytes()));
                }
                Err(_) => println!("    the serialized output failed to reparse"),
            }
        }
    }
}

/// The parse outcome for one input.
struct Outcome {
    parsed: bool,
    idempotent: bool,
}

/// Exercise every parse entry point on `input`, returning what happened. Panics
/// here are the primary bug oracle (caught by the caller).
fn exercise(input: &[u8]) -> Outcome {
    let mut idempotent = true;

    let parsed = match VcardCst::parse(input) {
        Ok(cst) => {
            let bytes = cst.to_bytes();
            let _ = cst.decode();
            let _ = cst.to_string();

            // Our own serialized output must reparse to a byte-stable fixpoint.
            match VcardCst::parse(&bytes) {
                Ok(reparsed) => idempotent = reparsed.to_bytes() == bytes,
                Err(_) => idempotent = false,
            }
            true
        }
        Err(_) => false,
    };

    for card in VcardCst::parse_many(input).flatten() {
        let _ = card.to_bytes();
        let _ = card.decode();
    }

    Outcome { parsed, idempotent }
}

/// Apply a few random byte edits to a seed.
fn mutate(seed: &[u8], rng: &mut Rng) -> Vec<u8> {
    let mut bytes = seed.to_vec();
    let edits = 1 + rng.below(16);

    for _ in 0..edits {
        if bytes.is_empty() {
            bytes.push(rng.byte());
            continue;
        }

        match rng.below(4) {
            0 => {
                let i = rng.below(bytes.len());
                bytes[i] = rng.byte();
            }
            1 => {
                let i = rng.below(bytes.len() + 1);
                bytes.insert(i, rng.byte());
            }
            2 => {
                let i = rng.below(bytes.len());
                bytes.remove(i);
            }
            _ => {
                let i = rng.below(bytes.len());
                bytes[i] ^= 1 << rng.below(8);
            }
        }
    }

    bytes
}

/// Render bytes readably: printable ASCII verbatim, everything else as `\xNN`.
fn escape(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &b in bytes {
        match b {
            b'\\' => out.push_str("\\\\"),
            0x20..=0x7e => out.push(b as char),
            b'\r' => out.push_str("\\r"),
            b'\n' => out.push_str("\\n"),
            other => out.push_str(&format!("\\x{other:02x}")),
        }
    }
    out
}

/// A tiny xorshift64 PRNG, so the run is deterministic and dependency-free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn byte(&mut self) -> u8 {
        self.next() as u8
    }
}
