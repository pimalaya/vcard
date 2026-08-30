# 📇 vcard-rs [![Documentation](https://img.shields.io/docsrs/vcard-rs?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/vcard-rs/latest/vcard) [![Coverage](https://img.shields.io/codecov/c/github/pimalaya/vcard/master?style=flat&logo=codecov&logoColor=white)](https://codecov.io/gh/pimalaya/vcard) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

vCard parser, validator, editor, merger and builder library for Rust

## Table of contents

- [Features](#features)
- [RFC coverage](#rfc-coverage)
- [Usage](#usage)
- [Examples](#examples)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- **All vCard versions**: parse and write 2.1, 3.0 and 4.0 through one version-agnostic model.
- **Byte-faithful editing**: change one parameter or value and every other byte of the card comes back unchanged, line endings included.
- **Forgiving parser**: accept any real card, even a malformed one, and round-trip it unchanged, its folds and blank lines included.
- **Strict building and validation**: construct cards checked against the standard, with an escape hatch to step outside it.
- **Three-way merge**: reconcile two divergent edits of a card against their common base, every action and conflict reported.
- **Small and portable**: no_std compatible, with an allocation-only core that pulls in no dependencies.
- **Optional content decoding**: quoted-printable text, inline base64 binary and foreign character sets, each behind its own feature.
- **Optional jCard**: read and write a card as JSON.
- **Optional JSContact**: convert a card to and from the Card object exchanged over JMAP.

> [!TIP]
> vcard-rs uses [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to gate optional support. The default feature set is declared in [Cargo.toml](./Cargo.toml) or on [docs.rs](https://docs.rs/crate/vcard-rs/latest/features).

## RFC coverage

| Spec   | What is covered                                                                                          |
|--------|----------------------------------------------------------------------------------------------------------|
| [2.1]  | vCard 2.1: the original versit format, including its quoted-printable and charset conventions              |
| [2425] | text/directory: bare directory records carrying no BEGIN and END envelope                                 |
| [2426] | vCard 3.0                                                                                                  |
| [6350] | vCard 4.0: the current standard, with its full property set, value types and parameters                   |
| [7095] | jCard: the JSON representation of a card                                                                   |
| [9553] | JSContact: the Card object model exchanged over JMAP                                                       |
| [9554] | vCard format extensions: the newer properties and the extended address components, modeled first-class    |
| [9555] | JSContact vCard mapping: the lossless conversion between a card and a JSContact Card                       |

[2.1]: https://www.imc.org/pdi/vcard-21.txt
[2425]: https://www.rfc-editor.org/rfc/rfc2425
[2426]: https://www.rfc-editor.org/rfc/rfc2426
[6350]: https://www.rfc-editor.org/rfc/rfc6350
[7095]: https://www.rfc-editor.org/rfc/rfc7095
[9553]: https://www.rfc-editor.org/rfc/rfc9553
[9554]: https://www.rfc-editor.org/rfc/rfc9554
[9555]: https://www.rfc-editor.org/rfc/rfc9555

## Usage

See documentation at [docs.rs](https://docs.rs/vcard-rs/latest/vcard).

## Examples

Every example is runnable with `cargo run --example <name>`, and prints what it does. See them all at [./examples](./examples).

| Example | What it shows |
|---------|---------------|
| [parse_edit_export](./examples/parse_edit_export.rs) | parse a card, rename the contact, write it back with every other byte untouched |
| [structured_values](./examples/structured_values.rs) | read and edit the components of `N`, `ADR` and a `CATEGORIES` list, one at a time |
| [forgiving_parse](./examples/forgiving_parse.rs) | a card no standard would bless (folds, blank lines, mixed endings, soft breaks) round-tripping byte for byte |
| [all_versions](./examples/all_versions.rs) | the same code reading a 2.1, a 3.0 and a 4.0 card, and where their values genuinely differ |
| [strict_builder](./examples/strict_builder.rs) | build a card checked against the RFC as it is assembled, then validated as a whole |
| [raw_builder](./examples/raw_builder.rs) | the escape hatch: assemble anything, checked by nothing |
| [validate_errors](./examples/validate_errors.rs) | every violation a card commits, and repairing one a strict server would reject |
| [three_way_merge](./examples/three_way_merge.rs) | reconcile two divergent edits against their base, with each side's actions and their conflicts |
| [address_book](./examples/address_book.rs) | walk a multi-card export, and read a bare RFC 2425 directory record |
| [content_encodings](./examples/content_encodings.rs) | quoted-printable, base64 and a foreign charset, kept raw then decoded on demand |
| [jcard](./examples/jcard.rs) | write a card as jCard JSON and read it back |
| [jscontact](./examples/jscontact.rs) | convert a card to a JSContact Card and back, escape hatches included |

The last three need their cargo feature: `--features quoted-printable,base64,encoding`, `--features jcard` and `--features jscontact`.

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

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
- 2026 → 2027: [NGI Zero Commons Fund](https://nlnet.nl/project/Pimalaya-pimdir/)

This program is part of Pimalaya, free software funded entirely by grants and donations. If you find it useful, consider [sponsoring](https://pimalaya.org/sponsor/) its development:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/pimalaya)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/pimalaya)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/pimalaya)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/u/gh/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
