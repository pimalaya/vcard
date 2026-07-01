# 📇 vcard-rs [![Documentation](https://img.shields.io/docsrs/vcard-rs?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/vcard-rs/latest/vcard-rs) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

Version-agnostic [vCard](https://www.rfc-editor.org/rfc/rfc6350) library in Rust.

`vcard-rs` is one decoded model and one byte-faithful syntax tree that read and write vCard 2.1 (versitcard), 3.0 ([RFC 2426](https://www.rfc-editor.org/rfc/rfc2426)) and 4.0 ([RFC 6350](https://www.rfc-editor.org/rfc/rfc6350)) alike. The card version is a decoded indicator, never a type parameter or a separate dialect: the syntax tree ignores it, and only the codec and the per-property spec branch on it where escaping or a value's shape genuinely differ. Parse raw bytes into the tree, edit one property, and every untouched byte round-trips.

```rust
use vcard::tree::cst::VcardCst;
use vcard::tree::prop::r#fn::FN;

let mut card = VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n").unwrap();

// read a property through its typed lens
assert_eq!(&*card.prop::<FN>().unwrap().0, "John Doe");

// edit it in place; every other byte is preserved
card.prop_mut::<FN>().unwrap().set_text("Jane Doe");
assert_eq!(card.to_string(), "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Jane Doe\r\nEND:VCARD\r\n");

// or project onto the version-agnostic decoded model
let decoded = card.decode();
```

## Table of contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [License](#license)
- [AI disclosure](#ai-disclosure)
- [Contributing](CONTRIBUTING.md)
- [Social](#social)
- [Sponsoring](#sponsoring)

## Features

- `no_std` (plus `alloc`); the core is dependency-free, the only dependencies are the small `no_std` crates behind the opt-in content-decoding features.
- **One model, every version**: a single decoded `Vcard` and a single parser for 2.1 / 3.0 / 4.0, with no per-version modules; the version is a value the syntax tree never consults.
- **Byte-faithful**: `VcardCst` reproduces the wire bytes exactly, so parse, edit and serialise round-trips byte for byte; a property value is kept as raw bytes, so a value in a foreign `CHARSET` survives (names and parameters must be UTF-8).
- **Liberal in, strict out** (Postel): one liberal parser accepts any real card, including unknown properties, parameters and value types; strictness is opt-in, as the spec-driven builder and `validate` (an RFC 6350 conformance check that still permits extensions).
- **Spec-driven**: each property carries its allowed versions, cardinality, value types and parameters per RFC 6350, consulted by the decoder, the validator and the builder alike.
- **Opt-in content decoders**, one small `no_std` crate per feature: `quoted-printable` (`=XX` octets), `base64` (inline binary values), and `encoding` (foreign `CHARSET` transcoding via [`encoding_rs`](https://crates.io/crates/encoding_rs)).

## Installation

```toml
[dependencies]
vcard-rs = "0.0.1"
```

## Usage

The crate has two layers. The decoded model is always available; the syntax tree is behind the default `parser` feature.

- Decoded model (`vcard`, `version`, `prop`, `param`, `value`): pure data with no dependency on the syntax side, so it can be depended on alone. A `Vcard` is a version plus a list of `VcardProp` (a name, parameters and one value); parameters and values are open payload enums with an `Unknown` arm, so anything outside the model survives.
- Syntax tree (`tree`): `tree::cst::VcardCst::parse` reads bytes or text (one card, a bare RFC 2425 record with no `BEGIN`/`END`, or every card via `parse_many`); `to_bytes` serialises byte-faithfully (`Display` / `to_string` is a lossy-for-non-UTF-8 convenience); `decode` projects onto the model and `encode` (or `From<Vcard>`) projects back; per-property lenses (`prop`, `prop_mut`) read and edit one line through byte-preserving cursors. The strict-out layer lives in `tree::vcard`: the `VcardPropBuilder` and `Vcard::validate`.
- Content decoders (behind `quoted-printable`, `base64`, `encoding`): the core surfaces a transfer-encoded or foreign-charset value raw, with its parameters kept, and these opt-in helpers decode it (`VcardValueCursor::quoted_printable` / `charset`, and `VcardBinary::decode_base64`).

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## AI disclosure

This project is developed with AI assistance. This section documents how, so users and downstream packagers can make informed decisions.

- **Tools**: Claude Code (Anthropic), Opus 4.8, invoked locally with a persistent project-scoped memory and a small set of repo-specific rules.
- **Used for**: Refactors, mechanical multi-file edits, boilerplate (feature gates, error enums, derive macros, trait impls), test scaffolding, doc polish, exploratory design conversations.
- **Not used for**: Engineering, critical code, git manipulation (commit, merge, rebase…), real-world tests.
- **Verification**: Every AI-assisted change is read, compiled, tested, and formatted before commit (`nix develop --command cargo check / cargo test / cargo fmt`). Behavioural correctness is verified against the relevant RFC or upstream spec, not assumed from the model output. Tests are never adjusted to fit AI-generated code; the code is adjusted to fit correct behaviour.
- **Limitations**: AI models occasionally produce code that compiles and passes tests but is subtly wrong: off-by-one errors, missed edge cases, plausible but nonexistent APIs, stale RFC references. The verification workflow catches most of this; it does not catch all of it. Bug reports are welcome and taken seriously.
- **Last reviewed**: 01/07/2026

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
