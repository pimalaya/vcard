---
cairn: log
change: a-group-rides-beside-the-name
landed: 2026-09-26
---

# A group rides beside the name

Follow-up to pimalaya/vcard#1. The lookups were fixed in [a-group-prefix-does-not-hide-a-property](./2026-09-26-a-group-prefix-does-not-hide-a-property.md), but the whole-card `decode` still parsed `item1.TEL` as an unknown name with a raw value, because the decoded model had nowhere to put the group. On an Apple or iCloud card most phones, emails, addresses and URLs were therefore invisible to typed consumers and to `validate`.

**`VcardProp::group`** (prop.rs) is an `Option<Cow<str>>`, kept verbatim. `VcardLine::decode` splits the wire name through the new `VcardLine::group` and `bare_name`, dispatching the value on the bare kind. `VcardProp::encode` joins them back, so the wire name round-trips.

**jCard** now reads the `group` parameter into the field instead of folding it into the name, and writes it from the field instead of splitting the name.

**JSContact** keeps its previous output: a grouped property goes to `vCardProps` whole. No Card member carries a group, and mapping `item1.TEL` to `phones` would cut it from its `item1.X-ABLabel` sibling.

**Breaking**: a new public field, so every `VcardProp` literal needs `group: None`; and a grouped property decodes as a `Kind` name where it used to be `Unknown("item1.TEL")`. The previous shape was also pinned by a jCard test, updated to the new one.

A side effect: `validate` matches on the kind, so a grouped property now has its cardinality, parameters and value checked like any other, where it used to be skipped as unknown.

Verified: 213 unit tests plus the corpus and coverage suites green, one new decode test and a grouped TEL added to the JSContact escape test. Clippy and rustdoc clean, and the bare core still builds dependency-free.

Spec updated: `decoded-model` (ADDED: "A group rides beside the name"; the model summary names the group).
