---
cairn: spec
capability: merge
status: current
---

# Three-way merge

`tree::merge::VcardMerge` reconciles two divergent copies of a card against their common base, building on the byte-preserving edit layer (see [editing](./editing.md)) so an untouched property keeps its bytes through a merge, bar the line ending a line gains when it stops being last.

### Requirement: Per-side action lists against a common base

The merge SHALL diff each side against the base into a list of `VcardMergeAction`, then replay the right side's actions onto a clone of the left.

The replay SHALL run in two phases: every edit of a value, a component or a parameter happens in place while no line moves, and the structural changes follow, removals first on descending indices, then each addition placed against the card as it then stands. Nothing addressed by index is read after an index has moved.

#### Scenario: Disjoint edits
- GIVEN a base card, a left copy editing `FN` and a right copy editing `EMAIL`
- WHEN they are merged
- THEN the result carries both edits and no conflict

#### Scenario: A removal above an edit
- GIVEN a right copy that removes the card's first property and edits three that follow it
- WHEN they are merged
- THEN every edit lands on the property it names and nothing is reported

### Requirement: The matching ladder

A property instance SHALL be matched across the three copies down one ladder, per name, and the rungs SHALL be consulted in this order:

1. The `PID` synchronisation identity (RFC 6350 section 7), with equality first, so two instances sharing one `PID` do not break the pair that needs no change, then `PID` alone.
2. The natural identity of a property that may occur more than once and whose value names a thing outside the card.
3. Identical serialized bytes, then equality alone, so among instances that decode alike the one that needs no rewrite is chosen.
4. Position.

A property whose value names a thing outside the card SHALL be identified by that value, the whole of it and as written: `EMAIL` by its address, `TEL` by its number, `IMPP`, `URL`, `SOURCE`, `FBURL`, `CALURI`, `CALADRURI`, `PHOTO`, `LOGO`, `SOUND`, `KEY` and `SOCIALPROFILE` by their URI, `MEMBER` and `RELATED` by the entity theirs names. Every other property SHALL carry no identity, since a property that may occur only once is already named by its name, and a property whose value is the datum would turn every edit into a replacement. A grouped name carries none either, the group being part of what tells the instance apart already.

`PID` SHALL stay above the natural identity. `PID` is metadata, so it survives a value change and a rename stays a rename, which an identity that is the value cannot do.

An identity SHALL tell an instance from its same-named siblings or it is not one: where two of them carry the same value, both fall back to their positions, and a sibling still alone with its value keeps its own. An instance carrying an identity SHALL NOT be matched with one carrying none, since the two are told apart differently and a position on one side does not answer for an identity on the other.

The identity SHALL be reported with the action, so a caller is told which member of a group is contested.

Where the identity is the value, changing the value changes the identity, so an edited address is one instance leaving and another arriving rather than a rename. Two sides that each replaced one therefore leave two instances, since neither renamed the base one and the name may repeat.

#### Scenario: Reordered instances
- GIVEN two copies whose `TEL` instances carry `PID` and appear in different orders
- WHEN they are merged
- THEN each instance is matched by its `PID`, not by its position

#### Scenario: Two instances under one PID
- GIVEN a base carrying two `TEL` instances under one `PID`, and a copy that edited one of them
- WHEN they are merged
- THEN the untouched instance keeps its bytes and only the edited one changes

#### Scenario: An interchangeable duplicate spelled two ways
- GIVEN a base carrying one property three times, one of them with a different line ending, and a copy carrying only the two identical ones
- WHEN they are merged
- THEN the copy the other two carry byte for byte survives

#### Scenario: A side that only reordered and replaced an address
- GIVEN a base holding two `EMAIL` instances, an untouched copy, and one that reordered them and replaced one address
- WHEN they are merged
- THEN the merged card is that copy, byte for byte, and nothing is reported

#### Scenario: An edit against a replacement
- GIVEN a copy that replaced Ada's `EMAIL` with Bob's and a copy that set a parameter on Ada's
- WHEN they are merged
- THEN Ada's parameter is never written onto Bob's line

#### Scenario: One address on two instances
- GIVEN a card carrying one `EMAIL` address twice, edited once
- WHEN it is merged with itself against the original
- THEN the edit lands, and nothing is reported

#### Scenario: A rename under a PID
- GIVEN a base `EMAIL;PID=1`, a copy adding a parameter and a copy changing the address
- WHEN they are merged
- THEN the card carries one `EMAIL`, with the new address and the new parameter

### Requirement: Matching normalises, writing is exact

An identity SHALL be compared normalised and written back exactly. The comparison lowercases, so a URI scheme (RFC 3986 section 3.1) and a mail host meet whichever case they were written in. What goes back on the wire is the bytes the side that wrote them wrote, never a normalised form the merge chose.

The two halves are one rule. Comparing raw bytes misses a match that is there; writing the normalised form loses the byte fidelity the whole crate is for.

A case difference in a value is still a change, since only matching normalises: a side that rewrote the case of a scheme rewrote the value, and that change lands like any other.

#### Scenario: One address in two cases
- GIVEN a base `IMPP:XMPP:ada@x.test`, a copy adding a parameter and a copy that lowercased the scheme
- WHEN they are merged
- THEN the card carries one `IMPP`, holding both parameters, spelled as the side that rewrote it spelled it

### Requirement: The base card is never mutated

The merge SHALL leave the base card untouched, and every position it carries SHALL be counted in that card. That is what makes the ladder's last rung safe: an ordinal read off the base names the same instance whenever it is resolved, however the merged card has moved under it.

The merged card is a clone of the left one and every edit lands on that clone, so nothing the merge does can renumber what a base ordinal names. A merge that mutated the base, or that numbered an action in the merged card, would need the ordinal translation ical-rs carries, which vcard-rs deliberately does not.

#### Scenario: An ordinal read off the base
- GIVEN a base carrying several same-named properties and a copy that removed one of them and edited another
- WHEN they are merged
- THEN the edit lands on the property its base ordinal names

### Requirement: Conflicts are reported, never silently resolved away

A divergent change to the same field on both sides SHALL be surfaced as a `VcardMergeConflict` in the `VcardMergeReport`, with the left side's action winning, except that an update beats a removal, at every granularity the merge diffs at: the whole property, one parameter, and one item of a list parameter alike.

A parameter an update restores over a removal is appended to the merged line, since the removal took its position with it and parameter order carries no meaning.

#### Scenario: Update against removal
- GIVEN a base property that the left copy removes and the right copy updates
- WHEN they are merged
- THEN the update wins and the conflict is reported

#### Scenario: A parameter update against a parameter removal
- GIVEN a base `TEL;PREF=1`, one copy dropping `PREF` and the other rewriting it to `PREF=2`
- WHEN they are merged in either order
- THEN the merged card carries `PREF=2` and one conflict is reported

### Requirement: Ours wins, and the collision is still reported

The left side SHALL be `ours` and the right side `theirs`, in git's sense. The merged card SHALL be built from the left side's bytes, and where both sides changed one field to different things it SHALL carry the left side's value. Neither is a caller's to choose.

One side answers both questions on purpose. A caller reaches for a merge holding the version it is merging into, and that version is the one it would rather not churn and the one it means to keep. Every collision is reported either way, so a caller wanting the other value resolves it from the report rather than asking the merge to guess.

The rule SHALL decide only the case where both sides wrote a value. An update still beats a removal whichever side it came from, a field one side alone touched is still taken from that side, an untouched line still comes out byte for byte, and the report still names both actions and the same fields.

A parameter or a property that loses SHALL NOT be written beside the one that beat it, so a name a version allows at most once is never written twice.

#### Scenario: A field one side alone touched
- GIVEN a field only the left copy changed, beside a field both copies changed
- WHEN they are merged
- THEN the left copy's change survives and nothing is reported for it

#### Scenario: A parameter both sides rewrote
- GIVEN a base `TEL;PREF=1`, a left copy holding `PREF=2` and a right copy holding `PREF=3`
- WHEN they are merged
- THEN the merged card carries `PREF=2` alone and the collision is reported

#### Scenario: Two additions of a name allowed at most once
- GIVEN a base without `UID`, and two copies each adding a different one
- WHEN they are merged
- THEN the merged card carries the left copy's `UID` alone and the collision is reported

### Requirement: Every change either lands or is reported

For every field of the merged card, the merge SHALL leave what one side made it and that side changed it, or what the base held and neither side changed it. A side's change that did not land SHALL appear in the `VcardMergeReport`'s conflicts, and no field SHALL hold a value neither side wrote, except a set-valued field, which carries both sides' additions and removals.

The field granularities are the ones the merge diffs at: the whole property, the whole value of a non-structured property, one component of a structured value, the item set of a list value, one parameter, and the item set of a list parameter.

#### Scenario: A change that cannot land
- GIVEN a base card and two copies that changed one field differently
- WHEN they are merged
- THEN the merged field holds one side's value, and the field is named in the conflicts

### Requirement: The merge obeys its algebraic laws

`merge(base, x, x)` SHALL yield `x` byte for byte and report nothing, `merge(base, x, base)` SHALL yield `x` byte for byte, `merge(base, base, y)` SHALL yield `y`, and `merge(base, base, base)` SHALL yield the base. The merged card SHALL parse again to a byte-stable fixpoint. Swapping the two sides SHALL report the same collided fields, and as many conflicts, though the merged bytes differ, since the left side's action wins. Re-merging the merged card against the base and either side SHALL change nothing.

#### Scenario: An untouched side
- GIVEN a base card and one copy that changed nothing
- WHEN they are merged in either order
- THEN the merged card is the other copy and nothing is reported

### Requirement: Values are compared on the raw node

The merge SHALL decide whether two copies hold the same value by comparing their raw value nodes component by component, over every component of the value, never through the decoded projection, which reads only a value's first `;`-component. An identity read off a value SHALL be read from the same raw node, for the same reason.

Two components agree when they decode to the same list of items, so a difference in escaping is a difference. An absent component and an all-empty one agree, so a trailing empty component is not a change.

Two parameters SHALL be compared the same way, on their raw nodes and value by value: a single-valued parameter decodes its first value alone, so two parameters differing past their first `,` decode alike and the edit is never reported. Where the two nodes carry different escaping modes they share no decoding to compare through, and only identical bytes are then certainly the same parameter. The replay SHALL address the right card's parameter node by name and ordinal, a decoded parameter not being a key either.

#### Scenario: A photo payload past the first semicolon
- GIVEN a base card carrying `PHOTO:data:image/png;base64,AAAA` and a copy carrying a different payload
- WHEN they are merged
- THEN the change is reported and lands, as one photo leaving and another arriving

#### Scenario: A title past the first semicolon
- GIVEN a base `TITLE:boss;of;nothing` and two copies rewriting what follows the second `;`
- WHEN they are merged
- THEN the divergence is reported

#### Scenario: A parameter past its first comma
- GIVEN a base `ADR;LABEL=Ada,Lovelace` and a copy holding `ADR;LABEL=Ada,Byron`
- WHEN they are merged
- THEN the change is reported and the merged card carries it

### Requirement: A whole-value change reports the whole value

The `old` and `new` payloads of a `ValueChanged` action SHALL say what the two raw value nodes say. A value whose decoded projection does not encode back to what its node holds SHALL be reported as the node's raw components (`VcardValue::Unknown`), since a non-structured value decodes its first `;`-component alone and would otherwise be reported truncated, with its old and its new equal. A value the model reads whole SHALL keep its decoded kind.

#### Scenario: A note changed past its first semicolon
- GIVEN a base and a left copy holding `NOTE:a;b`, and a right copy holding `NOTE:a;CHANGED`
- WHEN they are merged
- THEN the reported change holds both values whole, and the merged card carries the new one

### Requirement: Each parameter occurrence is a field of its own

One parameter name may be written more than once on a property (`TEL;TYPE=work;TYPE=voice`, RFC 2426 section 4), and the field a parameter action occupies SHALL be addressed by the parameter's key and by its position among the property's parameters of that key. Two sides editing two different occurrences of one name SHALL both land, uncontested.

Each parameter action SHALL carry that position, so a caller reading the report can tell one occurrence from another, and the replay SHALL resolve the occurrence the action names rather than the first parameter of that name.

#### Scenario: Two sides editing two parameters of one name
- GIVEN a base `TEL;TYPE=work;TYPE=voice`, a left copy rewriting the first `TYPE` and a right copy rewriting the second
- WHEN they are merged
- THEN the merged card carries both edits and nothing is reported

### Requirement: A change both sides made is never a conflict

Before pairing a right-side action with a colliding left one, the merge SHALL check whether any left action on the same base instance is the same change, and treat that as agreement: nothing to replay, nothing to report.

Two sides SHALL be held to have made one change only where they wrote the same bytes. An unescape is not injective, since `\N` and `\n` both read as a line break (RFC 6350 section 3.4), so two changes that decode alike may say different things on the wire, and reading those as one change drops the difference without a word. What is weighed is what the change itself wrote: the value it wrote, the `;`-component it wrote, the item a list gained, the parameter it wrote. A change that only takes something away wrote no bytes, and what it names lives in the base both sides share, so the change itself settles it.

The one exception SHALL be a list parameter the specification gives no order, `TYPE` (RFC 6350 section 5.6) and `PID` (section 7), whose items compare as a set both decoded and raw.

#### Scenario: A repeated parameter name
- GIVEN a base carrying `TEL;TYPE=work;TYPE=home` and two copies that both rewrote it to `TEL;TYPE=cell;TYPE=fax`
- WHEN they are merged
- THEN the merged card is that copy and nothing is reported

#### Scenario: One type set in two orders
- GIVEN a base whose `TEL` carries no `TYPE`, a left copy adding `TYPE=work,cell` and a right copy adding `TYPE=cell,work`
- WHEN they are merged
- THEN nothing is reported

#### Scenario: One value spelled two ways
- GIVEN a base holding `FN:Ada`, a left copy holding `FN:Ada\nLovelace` and a right copy holding `FN:Ada\NLovelace`
- WHEN they are merged
- THEN the divergence is reported and the merged card is the left copy's bytes

### Requirement: A replaced value outranks an item edit

A right-side item edit of a list value the left side replaced as a whole SHALL be reported as a conflict and dropped, so the merged value is the left side's replacement rather than a hybrid neither side wrote. Two item edits still merge as a set and never collide.

#### Scenario: An item added to a replaced list
- GIVEN a base `CATEGORIES:a,b`, a left copy replacing the whole value and a right copy adding one item
- WHEN they are merged
- THEN the merged value is the left copy's, and the collision is reported

### Requirement: The merged card is a card

Every line of the merged card but its last SHALL carry a line ending, so an addition after a line a source file left unterminated stays its own line and the merged card parses back to itself.

A line whose ending is already there keeps it verbatim, and a line that is still last keeps its empty one.

A merge that removes every line of a card yields no bytes at all, which the parser does not read back: an empty document is not a card, and the merge does not invent one to keep the fixpoint.

A bare record carrying a `BEGIN` or `END` line among its properties is degenerate in the same way: its bytes are read as an enveloped card the moment a `BEGIN` becomes the first line, so removing an earlier property changes what the same bytes describe. The merge preserves the content it was given, and the fixpoint requirement does not cover that shape either.

#### Scenario: An addition after an unterminated record
- GIVEN a base record `FN:a\r\nNOTE:b` read without a trailing break, and a right copy adding a `TEL`
- WHEN they are merged
- THEN the merged record holds three lines and reparses to three properties

#### Scenario: A record whose every property is removed
- GIVEN a bare record carrying one property and a right copy carrying only an `END` line
- WHEN they are merged
- THEN the merged record is empty, and the reparse law does not apply to it

### Requirement: An envelope line is not a property

The merge SHALL treat a `BEGIN` or `END` line among a card's lines as envelope rather than as a property: neither is diffed, matched, removed nor replayed, alongside the `VERSION` indicator. Replaying an `END` would otherwise close the merged card early and drop everything after it.

An addition SHALL be placed after the last line of the outer card sharing its name, never after a line of a card embedded in an `AGENT`, or at the end of the card when there is none. The embedded card's own lines are still diffed, so an edit to one of them lands and a divergent one is reported.

#### Scenario: An addition beside an embedded agent
- GIVEN a 2.1 card whose `AGENT` embeds a card carrying its own `FN`, and a copy adding an `FN` to the outer card
- WHEN they are merged
- THEN the added `FN` sits on the outer card, not on the agent

#### Scenario: A bare record carrying an END
- GIVEN a wrapped base card and a bare copy whose last line is `END:VCARD`
- WHEN they are merged
- THEN the merged card carries one `END` and reparses to itself

### Requirement: A replayed value carries its meaning, not its escaping

When the two cards escape values differently, which is vCard 2.1 against any later version, a value the merge replays from the right card SHALL be re-encoded for the merged card's escaping mode rather than copied byte for byte, so it keeps its meaning.

A value already written for the merged card's mode SHALL be copied unchanged, so a merge of one version's cards preserves its bytes.

Two cards escaping values by different rules share no decoding, so whether they hold the same value SHALL be decided on the raw bytes: the same line at two versions is not a change, and a line neither side touched is never rewritten.

#### Scenario: A 4.0 note replayed into a 2.1 card
- GIVEN a 2.1 base and left card carrying `NOTE:a,b` and a 4.0 right card carrying `NOTE:a\,c`
- WHEN they are merged
- THEN the merged 2.1 card carries the text `a,c`

### Requirement: A replayed parameter item keeps its wire form

An item the merge replays into a list parameter SHALL be written as the right card spelled it, never as its decoded text: a decoded item holds a real line break where the wire holds the RFC 6868 `^n`, so writing it back decoded would end the line in the middle of its head.

#### Scenario: A type value holding an encoded line break
- GIVEN a base `TEL;TYPE=work` and a right copy adding the item `a\nb^nc`
- WHEN they are merged
- THEN the merged line reads `TEL;TYPE=work,a\nb^nc` and parses back to itself

### Requirement: A removal both sides made takes one copy

A right-side item removal the left side already made SHALL not run again, so a list holding one item twice loses the one copy the two sides dropped rather than both.

#### Scenario: A repeated item both sides trimmed
- GIVEN a base carrying `NICKNAME:a,a` and two copies that both trimmed it to `NICKNAME:a`
- WHEN they are merged
- THEN the merged card is that copy

### Requirement: Byte preservation needs distinguishable instances

An untouched line SHALL keep its exact bytes through a merge whenever its instance can be told apart from the card's others. A card carrying the same property twice with the same content leaves the three matchings free to pair the copies differently, so which copy a removal takes, and which spelling survives, is not promised; the content is, by the completeness law.

#### Scenario: One property carried twice, identically
- GIVEN a card carrying one property twice with the same content and one copy removed on one side
- WHEN they are merged
- THEN one copy survives, and which of the two spellings it has is not pinned
