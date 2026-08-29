---
cairn: log
change: one-matching-ladder-for-both-crates
landed: 2026-08-29
---

# One matching ladder, the same in both crates

The matching now runs down the ladder both twins share: an explicit synchronisation identity, then a natural identity, then exact bytes and equality, then position. Only the per-format table differs, which is the point: the distance between vCard and iCalendar is data rather than a second design nobody can compare.

`PID` stays on the first rung and the natural identity goes on the second, in that order and not the other. `PID` is metadata, so it survives a value change and a rename stays a rename. An identity that *is* the value cannot do that, and inverting the two would throw away the one thing vCard has that iCalendar does not.

The table was settled by the audit's own test, may it repeat and does its value name something outside the card, applied to all 48 property kinds rather than to the list the campaign note sketched. Fifteen properties pass it: `EMAIL`, `TEL`, `IMPP`, `URL`, `SOURCE`, `FBURL`, `CALURI`, `CALADRURI`, `PHOTO`, `LOGO`, `SOUND`, `KEY`, `SOCIALPROFILE`, `MEMBER` and `RELATED`. The note proposed nine of them; the six additions are the repeatable URI-valued properties it did not enumerate, and leaving them out would have meant telling two properties apart by different mechanisms for no reason but which RFC section defines them. `GEO` is excluded although it is a URI in 4.0: a coordinate is the datum, like the `REQUEST-STATUS` iCalendar excludes. `CLIENTPIDMAP` is excluded because its identity is its first component rather than its value, and the merge already diffs it component by component. A grouped name carries no identity, following what `at_most_one` already does with one.

The three rules ical-rs learned from its fuzzer carry over. An identity a same-named sibling repeats is no identity, so both fall back and a sibling still alone with its value keeps its own. An instance carrying an identity is never matched with one carrying none, which is what guards the position rung. An addition numbered in the side that added it never meets an action numbered in the base, which vcard-rs already had structurally: an addition is routed by its own target rather than by a base ordinal, so the two numbering systems never meet.

The comparison discipline is now written down in both specs. Matching normalises, lowercasing so a URI scheme (RFC 3986 section 3.1) and a mail host meet whichever case they were written in. Writing is exact: what goes back on the wire is the bytes the side that wrote them wrote. Compare on raw bytes and a match is missed; write back the normalised form and the byte fidelity the crate exists for is gone. Only matching normalises, so a side that rewrote a scheme's case did change the value and that change lands like any other.

So is the invariant the positional rung has always leaned on. The base card is never mutated, so an ordinal counted in it names the same instance whenever it is resolved, however the merged card has moved. That is why vcard-rs needs none of the ordinal translation ical-rs carries, and it was load-bearing and unwritten.

The consequence the campaign note names is real and shows up in the tests. Where the identity is the value, changing the value changes the identity, so an edited address reads as one entry leaving and another arriving. Six tests moved for exactly that reason: each was about parameter replay, byte preservation or update-beats-removal, and each had been written with its value edit on a property that now renames rather than edits, which is a different subject. Three of them keep their `TEL` and move the other side's edit onto an `FN`; three swap the edited property for a `TITLE`, a `NOTE` or a `ROLE`. Two more state the new truth out loud: a photo payload replaced on both sides leaves two photos, and a divergence past a semicolon is still reported on a property whose value is not its identity. The generator and the corpus edit pool no longer rewrite the value of an identified property, as ical-rs's generator does not, since the reference keys a card by identity and cannot model a rename; the ladder has its own named tests instead.

`VcardPropPath` gained an `identity` field, mirroring `IcalPropPath`, so a report can name which member of a group is contested.

Capabilities moved: merge ("The matching ladder", "Matching normalises, writing is exact" and "The base card is never mutated" ADDED; "Values are compared on the raw node" MODIFIED; "Instance matching is PID, then equality, then position" REMOVED).
