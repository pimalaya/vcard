---
cairn: change
id: a-spec-is-not-syntax
status: landed
created: 2026-08-30
---

# A spec is not syntax

## Why

`tree::builder` reads as "builds a tree", and it does not: it builds a decoded card. `tree::validator` reads no better. The name was the symptom; the layering was the disease.

Both modules sat under `tree` because both are keyed on the per-property spec, and the spec sat on the lens markers, which sat under `tree`. Nothing about a spec is syntactic. It says which value kinds and which parameters a version allows a property to carry, which is a fact about the RFC, not about how bytes are laid out.

The cost of the misplacement was not cosmetic. `Vcard::validate`, `VcardBuilder`, `VcardPropBuilder` and both JSON codecs all reached through `prop_spec`, so all five required the `parser` feature and pulled in the tokeniser and `memchr`. Converting a decoded card to JSContact needed a parser it never called.

## What

Move the spec layer to the crate root: `prop::spec` (the trait, the vtable and its dispatch), `prop::cardinality`, and `param::COMMON_PARAMS`.

Split each of the 48 markers in two. The marker type and its `VcardPropSpec` impl go to `prop::<name>`; the `VcardPropLens` impl, the cursor and any bespoke codec stay at `tree::prop::<name>`. One type, two halves, each in the layer it belongs to, and rustdoc still shows both impls on the one type page.

Move `builder` and `validator` to the root behind them, and move the `From<VcardValid<Vcard>> for VcardCst` bridge next to its twin in `tree::codec::encode`, so no `#[cfg]` is needed anywhere.

Drop `parser` from the `jcard` feature.

## What this costs

48 property modules become 96. A reader asking what `EMAIL` is now has two files to look at: what the RFC allows, and how the crate reads and edits it. That is the price, and it is paid 48 times.

The alternative was to keep one file per property and make the root vtable a data table that the markers delegate to, which would move the single source of truth for a property's RFC facts off the marker and onto a 48-arm table. Keeping the marker as that source is worth the extra files.
