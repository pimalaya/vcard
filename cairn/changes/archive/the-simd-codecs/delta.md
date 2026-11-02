---
cairn: delta
change: the-simd-codecs
---

## ADDED Requirements

### Requirement: The CPU token is the application's

The decoders SHALL run the SIMD backend of the token the application installed through simd-cpu (`SimdCpu::install_detected()` in `main`, `SimdCpu::install` without std), and the baseline when none is installed. The crate SHALL NOT detect the CPU itself.

#### Scenario: No token installed
- GIVEN an application that never installs a token
- WHEN a base64 or quoted-printable value is decoded
- THEN it decodes under the baseline backend, to the same bytes a detected token gives

### Requirement: Decoding loses no byte

A decoder SHALL decode every value it is handed rather than fail it, and SHALL drop no byte silently: quoted-printable keeps every byte it cannot read as written, base64 skips a byte outside its alphabet and reports the payload as not well-formed.

#### Scenario: A raw byte in a quoted-printable value
- GIVEN a 2.1 `NOTE;ENCODING=QUOTED-PRINTABLE` value holding `caf=E9 \xFF`
- WHEN it is read through `quoted_printable`
- THEN it yields `caf\xE9 \xFF`

#### Scenario: An indented base64 continuation
- GIVEN a 2.1 `PHOTO;ENCODING=BASE64` whose continuation lines are indented by four spaces
- WHEN it is read through `decode_base64`
- THEN it yields the image bytes, the input reported well-formed

## MODIFIED Requirements

### Requirement: Decoding is feature-gated and composable

Each decoder SHALL sit behind its own cargo feature, backed by one `no_std` crate: `quoted-printable` for `=XX` octets (`VcardValueCursor::quoted_printable`, via quoted-printable-codec), `base64` for inline binary values (`VcardBinary::decode_base64`, via base64-codec), and `encoding` for a foreign `CHARSET` (`VcardValueCursor::charset`, via `encoding_rs`). The first two pull simd-cpu through their codec; without any of them the crate depends on nothing.

The charset helper composes the quoted-printable helper, so a value that is both quoted-printable and Latin-1 resolves in one call.

#### Scenario: Quoted-printable Latin-1
- GIVEN a 2.1 value declaring both `CHARSET=ISO-8859-1` and `ENCODING=QUOTED-PRINTABLE`, holding `caf=E9`
- WHEN it is read through `charset`
- THEN it yields the UTF-8 string `café`

## REMOVED Requirements
