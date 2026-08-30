---
cairn: log
change: a-module-is-read-before-it-is-run
date: 2026-08-30
---

# A module is read before it is run

Three files had outgrown a reader. `tree/merge.rs` held 2601 lines and about forty free functions, `jscontact.rs` 2534, `jcard.rs` 1049. Nothing was wrong with any of them, which is the point: correctness and coverage say nothing about whether a person can find the rule they are looking for.

The merge is now six modules, one per step of the work it does. `instance` decodes a card's property lines into the unit everything else addresses, `matching` pairs the base's instances with a side's down the identity ladder, `diff` reports what a side changed, `merger` replays the right side onto a clone of the left, `compare` holds the vocabulary all four decide sameness with, and `slot` the granularity at which two actions collide. `merge.rs` keeps the public types and the four-line pipeline that reads as the algorithm.

The free functions went with them, onto whatever they were about. `instances` became `Instance::all`, `identity_of`, `prop_path`, `line_eq` and `prop_eq` methods on `Instance`, `matching` became `Matching::new` over a private `Pairing` that owns one rung of the ladder at a time, `diff`/`diff_pair`/`diff_params` became `Diff`, and `value_eq`, `param_eq`, `param_key`, `unordered`, `param_alike`, `item_alike`, `raw_param_item`, `param_node`, `param_node_mut`, `transcode`, `terminate_lines`, `at_most_one` and `same_change` became methods on the nodes, lines, parameters, cards and kinds they were about. Five free functions are left, all genuinely functions of their arguments alone. `Instance` now carries the card it came from, which is what let the matching, the diff and the merger stop threading two `VcardCst` references through every call.

jCard is three modules, `export`, `import` and `datetime`, and its property, parameter and value conversions are `to_jcard` / `from_jcard` methods on `VcardProp`, `VcardParam`, `VcardValue` and `VcardValueUnknown` rather than eight free `*_to_jcard` and `*_from_jcard` functions.

JSContact is five: `export` holds the `Card` under construction, `import` the walk back, and `params`, `date` and `pointer` the three pieces both directions share. Its two big types keep their shape, since each is one cohesive object with one short method per JSContact member.

Every module header was distributed with the code it describes, so a parent now states the contract and a submodule its own rules. The merge header lost its matching ladder, its granularity table and its byte-equality rule to `matching`, `diff` and `compare`.

`tree::vcard` is gone. It held `builder` and `validate`, a noun over a noun and a verb, and nothing of its own; the two are now `tree::builder` and `tree::validator`. They stay under `tree` rather than moving to the crate root because both are keyed on the per-property spec, which is implemented on the lens markers, and a marker is as much a syntax key as it is a spec carrier.

The `NOTE:` audit cut 163 comments in `src/` to 79. What went was narration: a comment restating the assertion under it, or labelling a variable the code had already named. What stayed carries an RFC citation, a version-specific fact the assertion cannot show, or an invariant a reader would otherwise get wrong. Thirty-odd test comments describing a scenario became `///` docs on the test, which is where a scenario belongs and where `cargo test` can show it. Private structs and enums lost their per-field docs, since being private is the documentation, and `insts` is spelled `instances`.

No behaviour moved. The same 191 unit tests, the 37 merge laws over the corpus and the ten other suites pass unchanged, clippy is clean over all targets, the bare core still builds with no default features, and rustdoc emits no warnings.

Capabilities moved: conformance.
