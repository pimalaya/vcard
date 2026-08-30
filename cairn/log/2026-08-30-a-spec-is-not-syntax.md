---
cairn: log
change: a-spec-is-not-syntax
date: 2026-08-30
---

# A spec is not syntax

`tree::builder` reads as "builds a tree", which is not what it does. Chasing the name found the layering underneath it.

`VcardBuilder` and `Vcard::validate` sat under `tree` because both are keyed on the per-property spec, the spec sat on the lens markers, and the markers sat under `tree`. But nothing about a spec is syntactic: it says which value kinds and which parameters a version lets a property carry, which is a fact about RFC 6350, not about how bytes are folded. All 48 `VcardPropSpec` impls were checked before the move and not one of them touches a tree type.

The misplacement cost more than a name. `Vcard::validate`, both builders and both JSON codecs reached through `prop_spec`, so all five required the `parser` feature and compiled in the tokeniser and `memchr`. Converting a decoded card to JSContact needed a parser it never called. `jcard` is now `["dep:serde_json"]` alone, and the crate validates, builds and converts with no default features at all.

The spec layer moved to the root: `prop::spec` with its trait, its vtable and its dispatch, `prop::cardinality`, and `param::COMMON_PARAMS`. `builder` and `validator` followed it. The one thing holding them down was the `From<VcardValid<Vcard>> for VcardCst` bridge, which went to sit beside its twin `From<Vcard> for VcardCst` in `tree::codec::encode`, where it belonged anyway, so the move needed no `#[cfg]` anywhere.

Each of the 48 markers split in two. `prop::email::EMAIL` is the marker type and its `VcardPropSpec` impl, what the RFC allows. `tree::prop::email` keeps the `VcardPropLens` impl, the decode projection and the edit cursor, how the crate reads and edits it. Four markers carry a bespoke cursor (`N`, `ADR`, `GENDER`, `CLIENTPIDMAP`) and seven a bespoke decode (`GEO`, `KEY`, `LOGO`, `PHOTO`, `SOUND`, `SOCIALPROFILE`, and `AGENT`'s embedded card); all of that stayed on the tree side, where the prose describing it went too. Rustdoc still collects both impls onto the one type page, so what a reader looks up is unchanged.

`vcard::tree::prop::r#fn::FN` is now `vcard::prop::r#fn::FN`, which reads better and is the breaking part.

The price is 48 property modules becoming 96, paid once per property. The alternative was to keep one file each and turn the root vtable into a data table the markers delegate to, which would have moved the single source of truth for a property's RFC facts off the marker and onto a 48-arm table. The marker being that source is one of this crate's better ideas, so the files were the cheaper thing to spend.

288 tests pass, clippy is clean over all targets, rustdoc emits no warnings, and the bare core builds, as does `--no-default-features --features jscontact`, which could not be built before.

Capabilities moved: conformance.
