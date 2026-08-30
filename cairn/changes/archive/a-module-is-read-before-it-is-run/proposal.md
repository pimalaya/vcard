---
cairn: change
id: a-module-is-read-before-it-is-run
status: landed
created: 2026-08-30
---

# A module is read before it is run

## Why

Three files had grown past the point where a human can hold them: `tree/merge.rs` at 2601 lines, `jscontact.rs` at 2534, `jcard.rs` at 1049. Each was correct and each was covered, and none of that helps a reader who has to page through forty free functions to find where a value is compared.

The shape was the problem rather than the size. A free function has no context: `param_alike`, `same_change`, `raw_param_item` and `list_diff` sat at the same indentation as the type they were about, so nothing said which of them belonged together, and the module header had to carry every rule the file held because there was nowhere else to put them.

`NOTE:` comments had drifted the same way. 163 of them across `src/`, most in tests, most restating the assertion on the next line. A tag that appears everywhere stops marking anything.

And `tree::vcard` held `builder` and `validate`, a noun over a noun and a verb, and nothing of its own.

## What

Split the three files by domain, one submodule per step of the work, and attach every free function that has a `self` to the type it is about. Distribute each module header so that a submodule states its own rules and the parent states only the contract.

Read every `NOTE:` and keep the ones a competent reader would otherwise get wrong. A test whose comment describes its scenario carries a `///` doc instead, which is where a scenario belongs.

Flatten `tree::vcard` into `tree::builder` and `tree::validator`, the second renamed so both are nouns.

No behaviour moves: the same tests pass, unchanged.
