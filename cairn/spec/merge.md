---
cairn: spec
capability: merge
status: current
---

# Three-way merge

`tree::merge::merge` reconciles two divergent copies of a card against their common base, building on the byte-preserving edit layer (see [editing](./editing.md)) so an untouched property keeps its bytes through a merge, bar the line ending a line gains when it stops being last.

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

### Requirement: Instance matching is PID, then equality, then position

A property instance SHALL be matched across the three copies by its `PID` parameter and equality together first, then by `PID` alone (the RFC 6350 section 7 synchronisation identity), then by identical serialized bytes, then by equality alone, then by position, so a card that carries synchronisation identifiers merges by identity rather than by order, two instances sharing one `PID` do not break the pair that needs no change, and among interchangeable instances the one that needs no rewrite is chosen.

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

### Requirement: Conflicts are reported, never silently resolved away

A divergent change to the same field on both sides SHALL be surfaced as a `VcardMergeConflict` in the `VcardMergeReport`, with the left action winning, except that an update beats a removal, at every granularity the merge diffs at: the whole property, one parameter, and one item of a list parameter alike.

A parameter an update restores over a removal is appended to the merged line, since the removal took its position with it and parameter order carries no meaning.

#### Scenario: Update against removal
- GIVEN a base property that the left copy removes and the right copy updates
- WHEN they are merged
- THEN the update wins and the conflict is reported

#### Scenario: A parameter update against a parameter removal
- GIVEN a base `TEL;PREF=1`, one copy dropping `PREF` and the other rewriting it to `PREF=2`
- WHEN they are merged in either order
- THEN the merged card carries `PREF=2` and one conflict is reported

### Requirement: Every change either lands or is reported

For every field of the merged card, the merge SHALL leave what one side made it and that side changed it, or what the base held and neither side changed it. A side's change that did not land SHALL appear in the `VcardMergeReport`'s conflicts, and no field SHALL hold a value neither side wrote, except a set-valued field, which carries both sides' additions and removals.

The field granularities are the ones the merge diffs at: the whole property, the whole value of a non-structured property, one component of a structured value, the item set of a list value, one parameter, and the item set of a list parameter.

#### Scenario: A change that cannot land
- GIVEN a base card and two copies that changed one field differently
- WHEN they are merged
- THEN the merged field holds one side's value, and the field is named in the conflicts

### Requirement: The merge obeys its algebraic laws

`merge(base, x, x)` SHALL yield `x` byte for byte and report nothing, `merge(base, x, base)` SHALL yield `x` byte for byte, `merge(base, base, y)` SHALL yield `y`, and `merge(base, base, base)` SHALL yield the base. The merged card SHALL parse again to a byte-stable fixpoint. Swapping the two sides SHALL report the same collided fields, and as many conflicts, though the merged bytes differ, since the left action wins. Re-merging the merged card against the base and either side SHALL change nothing.

#### Scenario: An untouched side
- GIVEN a base card and one copy that changed nothing
- WHEN they are merged in either order
- THEN the merged card is the other copy and nothing is reported

### Requirement: Values are compared on the raw node

The merge SHALL decide whether two copies hold the same value by comparing their raw value nodes component by component, over every component of the value, never through the decoded projection, which reads only a value's first `;`-component.

Two components agree when they decode to the same list of items, so a difference in escaping is a difference. An absent component and an all-empty one agree, so a trailing empty component is not a change.

#### Scenario: A photo payload past the first semicolon
- GIVEN a base card carrying `PHOTO:data:image/png;base64,AAAA` and a copy carrying a different payload
- WHEN they are merged
- THEN the change is reported and lands, and two divergent payloads collide

### Requirement: A change both sides made is never a conflict

Before pairing a right-side action with a colliding left one, the merge SHALL check whether any left action on the same base instance is the same change, and treat that as agreement: nothing to replay, nothing to report.

Two actions are the same change when they are equal, except that a list parameter (`TYPE`, `PID`) compares its items as an unordered set, since RFC 6350 section 5.6 gives them no order.

#### Scenario: A repeated parameter name
- GIVEN a base carrying `TEL;TYPE=work;TYPE=home` and two copies that both rewrote it to `TEL;TYPE=cell;TYPE=fax`
- WHEN they are merged
- THEN the merged card is that copy and nothing is reported

#### Scenario: One type set in two orders
- GIVEN a base whose `TEL` carries no `TYPE`, a left copy adding `TYPE=work,cell` and a right copy adding `TYPE=cell,work`
- WHEN they are merged
- THEN nothing is reported

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

An item the merge replays into a list parameter SHALL be written as the right card spelled it, never as its decoded text: a parameter value is unescaped on the way in and copied verbatim on the way out, so a decoded item carrying a line break would end the line in the middle of its head.

#### Scenario: A type value holding an escaped line break
- GIVEN a base `TEL;TYPE=work` and a right copy adding the item `a\nb`
- WHEN they are merged
- THEN the merged line reads `TEL;TYPE=work,a\nb` and parses back to itself

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
