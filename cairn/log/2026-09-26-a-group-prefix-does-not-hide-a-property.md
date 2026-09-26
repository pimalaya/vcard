---
cairn: log
change: a-group-prefix-does-not-hide-a-property
landed: 2026-09-26
---

# A group prefix does not hide a property

Reported in pimalaya/vcard#1. `VcardLine::name` keeps the group prefix, and `prop`, `prop_mut`, `remove` and `fill_required` compared that whole name with the bare kind, so `item1.TEL` was never a `TEL`. Apple Contacts and iCloud group almost every TEL, EMAIL, ADR and URL, so on their cards the lenses found nothing, `remove` silently kept the grouped lines, and `fill_required` added an empty second `N` or `FN` next to a grouped one.

**`VcardLine::bare_name`** (tree/line.rs) returns the name after the group, and the four comparisons in tree/cst.rs match on it. The line itself is untouched, so the prefix round-trips as written.

Left as is: the whole-card `decode` still parses a grouped name as unknown, since the decoded model has no group field to carry the prefix. The merge already treats a grouped name as its own key, per the merge spec.

Verified: 212 unit tests plus the corpus and coverage suites green, one new test over an Apple-style card. Clippy clean, and the bare core still builds dependency-free.

Spec updated: `editing` (ADDED: "A group prefix does not hide a property").
