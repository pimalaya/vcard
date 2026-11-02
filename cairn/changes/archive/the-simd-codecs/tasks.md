---
cairn: tasks
change: the-simd-codecs
---

- [x] Settle the open questions with the user (the `decode_base64` signature, a checked quoted-printable reader, the release wait): agreed as proposed on 2026-11-02
- [x] Tests first: a 2.1 `PHOTO` with an indented base64 continuation, and a quoted-printable value carrying a raw 8-bit byte, each failing on the current decoders
- [x] Cargo.toml: base64-codec and quoted-printable-codec by git rev (d39f6b9, d63612b), simd-cpu f2eb72c as a dev-dependency, features `base64` and `quoted-printable` on them, `rust-version` 1.89; the fuzz crate alike
- [x] src/value/binary.rs: `decode_base64` over `Base64Codec::default().decode_vec`
- [x] src/tree/value/cursor.rs: `quoted_printable` over `QuotedPrintableCodec::default().decode_vec`, the raw fallback removed
- [x] Every corpus fixture carrying base64 or quoted-printable decodes alike under `SimdCpu::detect()` and `BASELINE` (tests/codecs.rs, a planted break in each call site caught)
- [x] fuzz/fuzz_targets/parse.rs decodes every binary and quoted-printable value
- [x] Docs: lib.rs header, the two method docs, README features, `install_detected` in the content_encodings example
- [x] `cargo test` under default, all and no features; clippy `--all-targets`; docs with `-Dwarnings`; `--no-default-features` dependency-free; thumbv6m-none-eabi and riscv32imc-unknown-none-elf builds with `base64,quoted-printable,parser`; `cargo fmt`
- [x] Fold content-encodings.md, log, CHANGELOG
