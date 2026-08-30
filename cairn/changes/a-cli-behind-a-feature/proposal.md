---
cairn: change
id: a-cli-behind-a-feature
status: active
created: 2026-08-30
---

# A CLI behind a feature

## Why

The crate is a library with no way to try it. Reaching for it today means writing a Rust file, and the first thing anyone wants from a vCard library is to point it at a `.vcf` and be told what is in there and what is wrong with it.

A small binary behind an off-by-default feature answers that at three moments. Someone evaluating the crate runs it once and sees the parse, the round trip and the violations without writing code. Someone reporting a bug pastes a command and its output instead of a repro crate. And the maintainer gets a way to run the codec over a file by hand, which today means a scratch example.

It also exercises the library the way a caller does. A CLI that cannot express something is evidence that the API cannot either.

The Pimalaya convention already covers the shape: a library carries its CLI as an off-by-default feature, and `pimalaya-cli` supplies the clap, printer and table pieces.

## What

A `cli` feature building a `vcard` binary over the existing API, with no new library surface.

Verbs, singular with plural hidden aliases:

- `parse` reads a card and prints its properties, or the parse error with its position.
- `validate` runs `Vcard::validate` and prints every violation at once, exiting non-zero when there is one.
- `convert` writes a card as jCard or JSContact, and reads either back.
- `build` assembles a card from repeated `--prop NAME=value` and `--param` arguments through `VcardBuilder`, so the spec refuses an illegal one at the point the argument is given.
- `merge` takes a base and two sides and prints the merged card, the actions and the conflicts.

Every data command needs its `*Output` type (`Display` + `Serialize` + `JsonSchema`) with a `--json` camelCase spelling, per the CLI output conventions.

Open questions to settle before starting:

- Whether `merge` belongs here at all, or whether it is tCard's job and the CLI should stop at the codecs.
- Whether the binary is named `vcard` (taken as a crate name concern) or `vcf`.
- Whether the `cli` feature is allowed to pull `std`, and how that reads against the crate's unconditional `no_std` core.

## What this is not

Not an editor. `tcard` already owns TOML-shaped editing, and this must not grow into a second one.
