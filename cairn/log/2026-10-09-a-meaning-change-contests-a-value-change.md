---
cairn: log
change: a-meaning-change-contests-a-value-change
landed: 2026-10-09
---

# A meaning change contests a value change

The twin of the ical-rs 0.6.0 change `a-zone-change-contests-a-time-change`, over a wider gap: no parameter contested a value change, `VALUE` included. `NOTE;LANGUAGE=en:Good morning` rewritten to `Good evening` on one side and relabelled `LANGUAGE=fr:Bonjour` on the other merged into `NOTE;LANGUAGE=fr:Good evening`, English labelled French, with only the text reported. A date moved on one side and given `CALSCALE=x-hebrew` on the other merged without a report.

`Slot::contests_meaning` (tree/merge/slot.rs) pairs a parameter `qualifies` names with every value-level slot, both ways, and `Merger::contest` (tree/merge/merger.rs) replaces `colliding`: it records the first left action on the right action's own field, as before, and every left action contesting what the value means. Each meaning pair is its own report so the count survives a swap of the sides; recording only the first, as the field rule does, made one parameter against two components one conflict one way and two the other. A parameter update over a removal is restored only where every collision is that removal, so a relabel the other side's text change contests no longer lands.

After an audit of RFC 6350 section 5, RFC 9554 and the 2.1 and 3.0 parameters the crate reads, the parameters are `VALUE`, `LANGUAGE`, `CALSCALE`, `MEDIATYPE`, `PHONETIC`, `SCRIPT`, `ENCODING`, `CHARSET` and the bare 2.1 `QUOTED-PRINTABLE`. Left out, as describing the thing rather than the value: `TYPE`, `PREF`, `ALTID`, `PID`, `SORT-AS`, `GEO`, `TZ`, `LABEL` and the other RFC 9554 parameters. `TYPE` carries a media format on a 3.0 `PHOTO`, `LOGO`, `SOUND` or `KEY`, whose value is its identity, so a changed one is a replacement that never meets it.

Two exemptions keep the rule symmetric and idempotent, both found by the property tests. A meaning action both sides made alike contests nothing, so a relabel both sides gave lets the translation one of them wrote land. A parameter removal the other side's update beats contests nothing either: without it, a dropped label against a relabelled rewrite restored the new label over the old text, and merging the result again then landed the text.

`a_list_item_edit_does_not_land_on_a_replaced_value` now counts two conflicts: the added item meets the replaced value and the `VALUE` that retyped it. The property tests' reference merge models the rule, and the completeness law lets a list a meaning change contested hold the left side's items. The base cards already carry `LANGUAGE` and the generator edits it, so the rule is exercised on every run; disabling it, or either exemption, fails the laws.

Capabilities moved: merge.
