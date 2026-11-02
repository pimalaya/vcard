---
cairn: log
change: the-simd-codecs
landed: 2026-11-02
---

# The SIMD codecs

Landed base64 and quoted-printable over the Pimalaya SIMD codecs, in place of the base64 0.23 and quoted_printable 0.5 crates, as mime-rs did the same day. Both decode under `SimdCpu::current()`, the token the application installed through simd-cpu or the baseline; the crate never detects the CPU. Pins: base64-codec d39f6b9, quoted-printable-codec d63612b, and simd-cpu f2eb72c as a dev-dependency (tests, the content_encodings example and the fuzz crate install the detected token), the revisions mime-rs pins, since two simd-cpu revisions in one build make two installed tokens. simd-cpu stays out of `[dependencies]`: the library only calls the codecs' `default()`, which read the token themselves. `rust-version` is 1.89.

`VcardBinary::decode_base64` now returns `Option<(Vec<u8>, bool)>`, the bytes and whether the payload was well-formed (agreed with the user on 2026-11-02), and decodes liberally: line breaks and blanks skipped, other bytes outside the alphabet skipped and reported, missing padding accepted. A 2.1 `PHOTO` with an indented continuation used to fail. `VcardValueCursor::quoted_printable` keeps its `Vec<u8>` and loses the raw fallback, the codec never failing; it used to drop every byte outside printable ASCII and the blanks before it (`caf=E9 \xFF` read `caf\xE9`). Both cases have a test in tests/encoding.rs that failed on the old decoders.

tests/codecs.rs installs the detected token and checks every quoted-printable and base64 value of the corpus decodes as the codecs' `BASELINE` does; a break planted in each call site was caught. The fuzz parse target installs the detected token, decodes every binary, quoted-printable and charset value, and checks a quoted-printable value never grows.

Verification: the suite under default, all and no features; clippy `--all-targets` under all and no features with `-D warnings`; docs with `-Dwarnings`; `cargo build --no-default-features` depends on nothing; thumbv6m-none-eabi and riscv32imc-unknown-none-elf build with `parser,base64,quoted-printable`; the parse fuzz target for 600 seconds (912,620 runs, coverage 3,722 to 5,521 from the corpus seed), no crash.

The crates.io release of vcard-rs waits on the codecs and simd-cpu being published (git dependencies block `cargo publish`).

Capabilities moved: content-encodings (ADDED: the CPU token is the application's, decoding loses no byte; MODIFIED: decoding is feature-gated and composable).
