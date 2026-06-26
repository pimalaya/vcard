# 📇 vcard-types [![Documentation](https://img.shields.io/docsrs/vcard-types?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/vcard-types/latest/vcard-types) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

Pure, parser-free [vCard](https://www.rfc-editor.org/rfc/rfc6350) model in Rust.

`vcard-types` is the in-memory representation of a vCard and nothing else: no separators, line folding, or escaping, which all belong to the parser. Build a card by plugging the public types together, every field is public and the aggregate is `Default`; parsing and serialising live in sibling crates.

```rust
use vcard_types::rfc6350::{Vcard, VcardProperty, VcardUriOrText};
use vcard_types::rfc6474::VcardBirthplace;

let mut card = Vcard {
    r#fn: vec![VcardProperty { value: "Jane Doe".into(), ..Default::default() }],
    email: vec![VcardProperty { value: "jane@example.org".into(), ..Default::default() }],
    ..Default::default()
};

// extension properties convert in with `From`
card.extensions.push(
    VcardBirthplace { value: VcardUriOrText::Text("Paris, France".into()), ..Default::default() }.into(),
);
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

- `no_std` (plus `alloc`), with no dependencies.
- **Complete** RFC 6350 vocabulary: every property, parameter and value type, organised by the spec's own sections under `rfc6350`.
- **Faithful** typing: each property carries exactly the value type the RFC permits, a concrete type when one is allowed, a small choice enum (`VcardUriOrText`, `VcardDateOrText`, `VcardTzValue`) when several are.
- **Extensible**: the extension RFCs (`rfc6474`, `rfc6715`, `rfc8605`, `rfc9554`, `rfc9555`) ship opt-in typed views that convert into one generic `VcardExtension` with `From`, so the core never depends on the supported set and there is no closed `Other` enum.
- **Parser-free**: the model holds decoded values only; grammar (folding, escaping, separators) belongs to the sibling `vcard-parser` and `vcard-builder` crates.

## Installation

```toml
[dependencies]
vcard-types = "0.0.1"
```

## Usage

The crate is data only; its modules mirror the specifications:

- `rfc6350`: the card (`Vcard`), the property and parameter wrappers (`VcardProperty`, `VcardParameters`), the section 4 value types, and the structured values grouped by the property sections (`VcardName`, `VcardAddress`, `VcardGender`, ...).
- `rfc6474`, `rfc6715`, `rfc8605`, `rfc9554`, `rfc9555`: typed views for each extension RFC, each with a `From<View> for VcardExtension`.

Build a `Vcard` from its public fields (see the example above), or receive one parsed by `vcard-parser`; serialise it back to vCard text with `vcard-builder`.

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
- **Last reviewed**: 26/06/2026

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
