---
cairn: change
id: the-simd-codecs
status: landed
created: 2026-11-02
---

# The SIMD codecs

## Why

mime-rs moved its base64 and quoted-printable to the Pimalaya SIMD codecs on 2026-11-02 (mime cairn: the-simd-codecs, the-installed-token, quoted-printable-over-the-codec), both reading the CPU token the application installs once through simd-cpu. vcard-rs still decodes through the base64 0.23 and quoted_printable 0.5 crates, scalar, and each with a rule at odds with this crate's liberal parsing:

- `VcardBinary::decode_base64` runs `BASE64_STANDARD`, which rejects any byte outside the alphabet. A 2.1 or 3.0 `PHOTO` whose continuation lines carry more than the one folding space the unfolder removes, or a stray blank, decodes to an error and the caller gets nothing.
- `VcardValueCursor::quoted_printable` runs `ParseMode::Robust`, which silently drops every byte outside printable ASCII (`hello\xFFworld` decodes to `helloworld`, the crate's own test). A 2.1 value mixing raw UTF-8 or Latin-1 with `=XX` octets loses its letters, against the content-encodings rule that nothing is silently lost.

Both codecs keep every byte, skip or keep what they cannot read, and report it, and they run AVX-512, AVX2, SSSE3 or NEON kernels under the installed token. One token across mime-rs and vcard-rs also means an application handling mail and contacts (io-pimdir, the Android app) detects the CPU once.

## What

- `base64` feature: base64-codec in place of base64, `decode_base64` decoding liberally under `SimdCpu::current()`: line breaks and blanks skipped, any other byte outside the alphabet skipped and reported, missing padding accepted.
- `quoted-printable` feature: quoted-printable-codec in place of quoted_printable, `quoted_printable()` and `charset()` decoding under `SimdCpu::current()` with the codec's rules: an `=` starting no valid sequence stays literal, raw 8-bit bytes and bare CRs pass through.
- simd-cpu reached through the codecs, a dev-dependency only (tests, examples and the fuzz crate install the detected token); `cargo build --no-default-features` stays dependency-free.
- Pins: base64-codec d39f6b9, quoted-printable-codec d63612b, simd-cpu f2eb72c, the revisions mime-rs pins. A different simd-cpu revision in one build makes two crates, so a token installed through one is never seen by the other and the second runs its baseline silently: vcard-rs and mime-rs must move their pins together.
- `rust-version` 1.88 to 1.89, the codecs' floor.
- Tests: the two data-loss cases above fail before and pass after; a detected token decodes as `BASELINE` does on every fixture of the corpus carrying `ENCODING=b`, `ENCODING=BASE64` or `QUOTED-PRINTABLE`; the fuzz crate's parse target also decodes every binary and quoted-printable value.
- Docs: lib.rs header, the module docs of src/value/binary.rs and src/tree/value/cursor.rs, the README features table, a line telling applications to call `SimdCpu::install_detected()` in `main`.

Out of scope: an encoder path (the crate writes neither base64 nor quoted-printable today), the parser's own scanning (see [the-lines-built-in-place](../../the-lines-built-in-place/proposal.md)), and the consumers' `install_detected` calls, each its own change.

## Open questions

- `decode_base64` returns `Option<Result<Vec<u8>, base64::DecodeError>>`, the error type of a crate leaving the API, so the signature breaks whatever is chosen. Proposed: `Option<(Vec<u8>, bool)>`, the bytes always and whether the input was well-formed, the shape mime-rs's readers return. The alternative, a vcard-rs error type, would throw away the bytes the liberal decoder recovered.
- `quoted_printable()` keeps returning `Vec<u8>`: the codec never fails, so the fallback to raw bytes goes. A `quoted_printable_checked()` returning the clean flag beside the bytes is cheap to add; proposed only if a consumer asks.
- Publishing: vcard-rs is on crates.io (0.5.2) and git-rev dependencies block `cargo publish`, so the next vcard-rs release waits on the codecs and simd-cpu being published, as mime-rs's first release does.
