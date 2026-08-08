---
cairn: spec
capability: content-encodings
status: current
---

# Content encodings

How the crate treats a content transfer encoding (`QUOTED-PRINTABLE`, `BASE64`) and a foreign character set (`CHARSET`). The rule is that the core transforms no content; decoding is opt-in, one small `no_std` crate per cargo feature.

The line between "parsing" and "content decoding" is deliberate. Resolving the value grammar (escapes and line folding) is parsing, so the core does it. Turning `=E9` into a character, or Latin-1 bytes into UTF-8, is content decoding, so the core leaves it alone.

### Requirement: The core keeps encoded content raw

A `QUOTED-PRINTABLE` or `BASE64` transfer encoding and a `CHARSET` SHALL be surfaced as raw bytes with their parameters kept, so nothing is silently lost or transcoded.

An earlier design where the core quoted-printable-decoded values was reverted: it ran a lossy UTF-8 conversion that silently destroyed foreign-charset bytes.

#### Scenario: A quoted-printable value read from the core
- GIVEN a line declaring `ENCODING=QUOTED-PRINTABLE`
- WHEN its value is read through the cursor's `bytes`
- THEN the `=XX` octets come back unresolved, and the `ENCODING` parameter is still on the line

### Requirement: Decoding is feature-gated and composable

Each decoder SHALL sit behind its own cargo feature, backed by one `no_std` crate: `quoted-printable` for `=XX` octets (`VcardValueCursor::quoted_printable`), `base64` for inline binary values (`VcardBinary::decode_base64`), and `encoding` for a foreign `CHARSET` (`VcardValueCursor::charset`, via `encoding_rs`).

The charset helper composes the quoted-printable helper, so a value that is both quoted-printable and Latin-1 resolves in one call.

#### Scenario: Quoted-printable Latin-1
- GIVEN a 2.1 value declaring both `CHARSET=ISO-8859-1` and `ENCODING=QUOTED-PRINTABLE`, holding `caf=E9`
- WHEN it is read through `charset`
- THEN it yields the UTF-8 string `café`
